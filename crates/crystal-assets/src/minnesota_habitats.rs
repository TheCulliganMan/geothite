//! Authored local wildlife habitats on existing native grass and Surf water.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{
    WildEncounter, WildEncounterData, WildEncounterTable, WildEncounterZone,
};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_surface::{reachable_map_with_surf, walking_map_distances};
use crate::authored_town_geometry::kind;

const MANIFEST: &str = "minnesota-species-habitats-v2";
const TIMES: [&str; 3] = ["morning", "day", "night"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u8,
    id: String,
    scope: String,
    grass: Vec<Grass>,
    water: Vec<Water>,
    targets: BTreeMap<String, String>,
    deferred: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Grass {
    id: String,
    map: String,
    rect: [u16; 4],
    anchor: [u16; 2],
    theme: String,
    level: u8,
    residents: BTreeMap<String, Vec<String>>,
    rare: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Water {
    id: String,
    map: String,
    time: String,
    slot: usize,
    species: String,
    level: u8,
    anchor: [u16; 2],
    sample: [u16; 2],
}

/// Installs 40 ordinary-family root acquisition paths in 14 native grass habitats
/// and seven Surf tables. Grass fallback tables, terrain, actors, scripts,
/// parties and audio are preserved. New private Good/Super Rod groups retain
/// original groups, Old Rod tables, thresholds, timed slots and swarm behavior.
/// Legendary, fossil, fair and runestone discoveries remain explicitly separate
/// authored work.
///
/// Static walking/Surf checks establish local surface access with the relevant
/// field prerequisites assumed. They do not prove earned campaign progression.
///
/// # Errors
/// Rejects an incomplete plan, duplicate installation, inactive/non-root or
/// legendary species, unsafe/overlapping habitats, inaccessible samples,
/// unavailable time/rate tables, lost previous species or invalid native output.
/// The borrowed parent is unchanged on failure.
pub fn build_minnesota_habitats(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "habitats already installed"
    );
    let source: Source = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1
            && source.id == MANIFEST
            && !source.scope.is_empty()
            && source.grass.len() == 14
            && source.water.len() == 7
            && source.targets.len() == 40
            && source.deferred.is_object(),
        "complete authored habitat plan required"
    );
    let policy = crate::authored_regional_approach::policy()?;
    let roster: Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/pokemon/roster.json"
    ))?;
    let mut targets = BTreeSet::new();
    for (family, species) in &source.targets {
        let record = roster["families"]
            .as_array()
            .context("native families")?
            .iter()
            .find(|f| f["id"].as_str() == Some(family))
            .context("target family")?;
        ensure!(
            record["rarity"] != "legendary" && record["rarity"] != "mythical",
            "rare discovery cannot become ordinary wildlife"
        );
        ensure!(
            record["members"][0]["name"]
                .as_str()
                .is_some_and(|n| n.to_ascii_uppercase() == *species)
                && policy.allowed.contains(species)
                && targets.insert(species.clone()),
            "active unique family root required: {species}"
        );
    }
    let mut placed = BTreeSet::new();
    let mut pack = base.clone();
    let mut ids = BTreeSet::new();
    let mut covered: BTreeMap<String, BTreeSet<[u16; 2]>> = BTreeMap::new();
    for h in source.grass {
        ensure!(
            ids.insert(h.id.clone())
                && h.map.starts_with("Mn")
                && !h.theme.is_empty()
                && h.residents
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>()
                    == BTreeSet::from(TIMES)
                && h.rare.keys().all(|t| TIMES.contains(&t.as_str())),
            "habitat identity/time differs: {}",
            h.id
        );
        let map = base.data.maps.get(&h.map).context("habitat map")?;
        let occupied = map.objects.iter().map(|o| [o.x, o.y]).collect();
        let accessible = walking_map_distances(base, map, h.anchor, &occupied)?;
        let [left, top, right, bottom] = h.rect;
        ensure!(
            left < right
                && top < bottom
                && right < map.attributes.width * 2
                && bottom < map.attributes.height * 2,
            "bounded native grass rectangle required"
        );
        let cells = covered.entry(h.map.clone()).or_default();
        for y in top..=bottom {
            for x in left..=right {
                ensure!(
                    kind(base, map, [x, y])? == "TALL_GRASS"
                        && accessible.contains_key(&[x, y])
                        && cells.insert([x, y]),
                    "{} requires reachable nonoverlapping native grass at {x},{y}",
                    h.id
                );
            }
        }
        let data = pack
            .data
            .wild_encounters
            .get_mut(&h.map)
            .context("base grass encounters")?;
        ensure!(
            data.zones
                .iter()
                .all(|z| i32::from(right) < i32::from(z.min_x)
                    || i32::from(left) > i32::from(z.max_x)
                    || i32::from(bottom) < i32::from(z.min_y)
                    || i32::from(top) > i32::from(z.max_y)),
            "habitat overlaps an installed zone"
        );
        let rates = data.grass_rates.clone().context("native grass rates")?;
        ensure!(
            TIMES.iter().all(|t| rates.get(*t).is_some_and(|r| *r > 0)),
            "inactive native grass time"
        );
        let mut grass = data.grass.clone().context("base grass table")?;
        for time in TIMES {
            let slots = slots_mut(&mut grass, time)?;
            ensure!(slots.len() == 7, "seven native grass slots required");
            let min = slots
                .iter()
                .map(|s| s.level)
                .min()
                .context("grass levels")?;
            let max = slots
                .iter()
                .map(|s| s.level)
                .max()
                .context("grass levels")?;
            ensure!(
                (min..=max).contains(&h.level),
                "habitat level exceeds local native range"
            );
            let residents = &h.residents[time];
            ensure!(
                (1..=2).contains(&residents.len()),
                "one or two time-specific residents required"
            );
            for species in residents.iter().chain(h.rare.get(time)) {
                ensure!(
                    targets.contains(species)
                        && policy.target(base, species, Some(h.level))? == *species,
                    "unapproved wildlife: {species}"
                );
                placed.insert(species.clone());
            }
            for (index, resident) in [
                (2, 0),
                (3, residents.len() - 1),
                (5, 0),
                (6, residents.len() - 1),
            ] {
                slots[index] = WildEncounter {
                    species: residents[resident].clone(),
                    level: h.level,
                };
            }
            if let Some(rare) = h.rare.get(time) {
                slots[6] = WildEncounter {
                    species: rare.clone(),
                    level: h.level,
                };
            }
        }
        data.zones.push(WildEncounterZone {
            id: format!("MN_HABITAT_{}", h.id.replace('-', "_").to_ascii_uppercase()),
            min_x: i16::try_from(left)?,
            min_y: i16::try_from(top)?,
            max_x: i16::try_from(right)?,
            max_y: i16::try_from(bottom)?,
            grass_rates: rates,
            grass,
        });
    }
    for name in covered.keys() {
        let map = &base.data.maps[name];
        let mut remaining = 0;
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                if pack.data.wild_encounters[name]
                    .zone_at(i16::try_from(x)?, i16::try_from(y)?)
                    .is_none()
                    && kind(base, map, [x, y])? == "TALL_GRASS"
                {
                    remaining += 1;
                }
            }
        }
        ensure!(
            remaining >= 4,
            "{name} needs native fallback grass outside new habitats"
        );
    }
    for h in source.water {
        ensure!(
            ids.insert(h.id)
                && h.map.starts_with("Mn")
                && TIMES.contains(&h.time.as_str())
                && h.slot == 1
                && targets.contains(&h.species)
                && policy.target(base, &h.species, Some(h.level))? == h.species,
            "authored native water resident required"
        );
        let map = base.data.maps.get(&h.map).context("water habitat map")?;
        let occupied = map.objects.iter().map(|o| [o.x, o.y]).collect();
        let accessible = reachable_map_with_surf(base, map, h.anchor, &occupied)?;
        let water_kind = kind(base, map, h.sample)?;
        ensure!(
            (water_kind == "WATER" || water_kind.starts_with("WATER_"))
                && accessible.contains(&h.sample),
            "reachable Surf sample required: {}",
            h.map
        );
        let data = pack
            .data
            .wild_encounters
            .get_mut(&h.map)
            .context("native water encounters")?;
        ensure!(
            data.water_rate.is_some_and(|r| r > 0),
            "active water rate required"
        );
        let table = data.water.as_mut().context("native Surf table")?;
        let slots = slots_mut(table, &h.time)?;
        ensure!(slots.len() == 3, "three native water slots required");
        let min = slots
            .iter()
            .map(|s| s.level)
            .min()
            .context("water levels")?;
        let max = slots
            .iter()
            .map(|s| s.level)
            .max()
            .context("water levels")?;
        ensure!(
            (min..=max).contains(&h.level),
            "water level exceeds local native range"
        );
        slots[h.slot] = WildEncounter {
            species: h.species.clone(),
            level: h.level,
        };
        crate::minnesota_habitat_fishing::install(&mut pack, &h.map, &h.species, h.level)?;
        placed.insert(h.species);
    }
    ensure!(
        placed == targets,
        "every declared family needs an actual encounter slot"
    );
    for (name, original) in &base.data.wild_encounters {
        let after = &pack.data.wild_encounters[name];
        ensure!(
            species(original).is_subset(&species(after)),
            "lost previous wildlife on {name}"
        );
        if name.starts_with("Mn") {
            ensure!(
                species(after).iter().all(|s| policy.allowed.contains(s)),
                "foreign or retired regional wildlife on {name}"
            );
        } else {
            ensure!(original == after, "unrelated region changed");
        }
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn slots_mut<'a>(
    table: &'a mut WildEncounterTable,
    time: &str,
) -> Result<&'a mut Vec<WildEncounter>> {
    match time {
        "morning" => Ok(&mut table.morning),
        "day" => Ok(&mut table.day),
        "night" => Ok(&mut table.night),
        _ => anyhow::bail!("unknown native encounter time"),
    }
}

fn species(data: &WildEncounterData) -> BTreeSet<String> {
    data.grass
        .iter()
        .chain(&data.water)
        .chain(data.zones.iter().map(|z| &z.grass))
        .chain(data.swarm_overrides.values().map(|s| &s.grass))
        .flat_map(|t| t.morning.iter().chain(&t.day).chain(&t.night))
        .map(|s| s.species.clone())
        .collect()
}

#[cfg(test)]
#[path = "minnesota_habitats_tests.rs"]
mod tests;
