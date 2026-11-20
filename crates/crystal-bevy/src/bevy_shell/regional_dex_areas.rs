//! Wild habitats on the Minnesota atlas, collected once when opening AREA.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use crystal_assets::{GameDataSet, MapModule, SafariZone};
use crystal_core::state::RoamingPokemonState;
use crystal_core::world::collision::{
    Terrain, TilesetCollision, describe_collision, is_grass_encounter_permission, permissions,
};
use crystal_core::world::encounters::{TimeOfDay, WildEncounterData, WildEncounterTable};
use crystal_core::world::fishing::{FISHING_RODS, FishingCatalog, RodTable};
use crystal_core::world::map::TilePosition;

use super::minnesota_atlas_routes;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct Area {
    pub name: String,
    pub x: i16,
    pub y: i16,
    pub method: String,
    pub time: &'static str,
}

const TIMES: [TimeOfDay; 3] = [TimeOfDay::Morning, TimeOfDay::Day, TimeOfDay::Night];
const ALL: u8 = 7;

fn time_label(mask: u8) -> &'static str {
    match mask {
        1 => "MORNING",
        2 => "DAY",
        3 => "MORN/DAY",
        4 => "NIGHT",
        5 => "MORN/NIGHT",
        6 => "DAY/NIGHT",
        _ => "",
    }
}

fn table_times(table: &WildEncounterTable, species: &str, rate: impl Fn(&str) -> u8) -> u8 {
    let mut times = 0;
    for (index, time) in TIMES.iter().copied().enumerate() {
        if rate(time.as_key()) > 0 && table.slots(time).iter().any(|s| s.species == species) {
            times |= 1 << index;
        }
    }
    times
}

#[derive(Default)]
struct Surfaces<'a> {
    grass: bool,
    water: bool,
    tree: bool,
    zones: BTreeSet<&'a str>,
}

fn surfaces<'a>(
    map: &MapModule,
    tiles: &TilesetCollision,
    wild: Option<&'a WildEncounterData>,
) -> Surfaces<'a> {
    let mut found = Surfaces::default();
    let width = usize::from(map.attributes.width);
    let indoors = matches!(
        map.attributes.environment.as_deref(),
        Some("CAVE" | "DUNGEON")
    );
    for (index, &block) in map.blocks.iter().enumerate() {
        let Some(tile) = tiles.metatiles.get(usize::from(block)) else {
            continue;
        };
        for (quadrant, &kind) in tile.collision.iter().enumerate() {
            let terrain = describe_collision(kind).terrain;
            found.water |= terrain == Terrain::Water;
            found.tree |= matches!(
                kind,
                permissions::HEADBUTT_TREE | permissions::HEADBUTT_TREE_1D
            );
            if terrain != Terrain::Land || matches!(kind, permissions::ICE | permissions::ICE_2B) {
                continue;
            }
            let x = (index % width * 2 + quadrant % 2) as i16;
            let y = (index / width * 2 + quadrant / 2) as i16;
            if let Some(zone) = wild.and_then(|w| w.zone_at(x, y)) {
                found.zones.insert(&zone.id);
            } else {
                found.grass |= indoors || is_grass_encounter_permission(kind);
            }
        }
    }
    found
}

fn fishing_times(catalog: &FishingCatalog, table: &RodTable, species: &str) -> u8 {
    let mut previous = None;
    let mut times = 0;
    for slot in &table.slots {
        if previous == Some(slot.threshold) {
            continue;
        }
        previous = Some(slot.threshold);
        if let Some(fish) = &slot.species {
            if fish == species {
                return ALL;
            }
        } else if let Some(entry) = slot
            .time_group
            .as_ref()
            .and_then(|id| catalog.time_groups.get(id))
        {
            if entry.day_species == species {
                times |= 3;
            }
            if entry.night_species == species {
                times |= 4;
            }
        }
    }
    times
}

fn safari_tiles<'a>(
    data: &GameDataSet,
    map: &MapModule,
    zone: &'a SafariZone,
    tiles: &TilesetCollision,
) -> Result<BTreeSet<(&'a str, u8)>> {
    let mut sites = BTreeSet::new();
    let width = usize::from(map.attributes.width);
    for (index, &block) in map.blocks.iter().enumerate() {
        let tile = tiles
            .metatiles
            .get(usize::from(block))
            .context("Safari metatile")?;
        for (quadrant, &kind) in tile.collision.iter().enumerate() {
            if !matches!(kind, permissions::TALL_GRASS | permissions::WATER) {
                continue;
            }
            let x = i16::try_from(index % width * 2 + quadrant % 2)?;
            let y = i16::try_from(index / width * 2 + quadrant / 2)?;
            // The encounter dispatcher uses this exact tile and excludes the free
            // entry court and indoor stops, even where the same tileset is used.
            if let Some(habitat) = zone.habitat_at(data, &map.id, TilePosition::new(x, y))? {
                sites.insert((habitat.id.as_str(), kind));
            }
        }
    }
    Ok(sites)
}

// Combine chunks and repeated slots at the same atlas stop, keeping the encounter method.
pub(super) fn collect(
    data: &GameDataSet,
    species: &str,
    roaming: &[RoamingPokemonState],
) -> Result<Vec<Area>> {
    let stops = minnesota_atlas_routes::stops(&data.pokegear_landmarks);
    let mut collisions = BTreeMap::new();
    let mut rows = BTreeMap::new();
    // Crystal's nest screen consults only the first two active slots. A newly
    // released roamer can have zero stored HP until its first encounter.
    let roaming_maps: BTreeSet<_> = roaming
        .iter()
        .take(2)
        .filter(|slot| slot.species.as_deref() == Some(species))
        .filter(|_| data.minnesota_dex_members().any(|member| member == species))
        .filter_map(|slot| {
            data.runtime_map_metadata.values().find(|map| {
                map.group_id == u16::from(slot.map_group)
                    && map.map_id == u16::from(slot.map_number)
            })
        })
        .map(|map| map.name.as_str())
        .collect();
    for (map_id, map) in &data.maps {
        if !map_id.starts_with("Mn") {
            continue;
        }
        let Some(place) = &map.attributes.location else {
            continue;
        };
        let Some(cursor) = minnesota_atlas_routes::player_cursor(&stops, map_id, place) else {
            continue;
        };
        let stop = stops
            .iter()
            .find(|s| s.cursor == cursor)
            .context("Minnesota atlas stop")?;
        let name = stop
            .route
            .map_or_else(|| stop.landmark.name.clone(), |n| format!("ROUTE {n}"));
        let tiles = match collisions.entry(&map.attributes.tileset_name) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(data.tileset_collision(&map.attributes.tileset_name)?)
            }
        };
        let wild = data.wild_encounters.get(map_id);
        let surface = if let Some(safari) = data
            .safari_zone
            .as_ref()
            .filter(|z| z.maps.contains_key(map_id))
        {
            let sites = safari_tiles(data, map, safari, tiles)?;
            for habitat in &safari.habitats {
                for (kind, rate, table, method) in [
                    (
                        permissions::TALL_GRASS,
                        habitat.grass_rate,
                        &habitat.grass,
                        "SAFARI GRASS",
                    ),
                    (
                        permissions::WATER,
                        habitat.water_rate,
                        &habitat.water,
                        "SAFARI WATER",
                    ),
                ] {
                    if rate > 0
                        && sites.contains(&(habitat.id.as_str(), kind))
                        && table
                            .iter()
                            .any(|slot| slot.species == species && slot.weight > 0)
                    {
                        let name = match habitat.id.as_str() {
                            "PINE_PORTAGE_MIDDLE" => "MIDDLE PINE PORTAGE".into(),
                            "PINE_PORTAGE_SOUTH" => "SOUTH PINE PORTAGE".into(),
                            id => id.replace('_', " "),
                        };
                        rows.insert(
                            (name, stop.landmark.x, stop.landmark.y, method.to_owned()),
                            ALL,
                        );
                    }
                }
            }
            // Rods still use native time and swarm tables inside the preserve.
            Surfaces {
                water: sites.iter().any(|(_, kind)| *kind == permissions::WATER),
                ..Default::default()
            }
        } else {
            surfaces(map, tiles, wild)
        };
        let mut add = |method: &str, time: u8| {
            if time != 0 {
                *rows
                    .entry((
                        name.clone(),
                        stop.landmark.x,
                        stop.landmark.y,
                        method.to_owned(),
                    ))
                    .or_insert(0) |= time;
            }
        };
        if roaming_maps.contains(map_id.as_str()) {
            add("ROAMING", ALL);
        }
        if let Some(wild) = wild {
            let land = if matches!(
                map.attributes.environment.as_deref(),
                Some("CAVE" | "DUNGEON")
            ) {
                "CAVE"
            } else {
                "GRASS"
            };
            if surface.grass {
                if let Some(table) = &wild.grass {
                    add(
                        land,
                        table_times(table, species, |t| {
                            wild.grass_rates
                                .as_ref()
                                .and_then(|r| r.get(t))
                                .copied()
                                .unwrap_or(0)
                        }),
                    );
                }
                for swarm in wild.swarm_overrides.values() {
                    add(
                        "GRASS SWARM",
                        table_times(&swarm.grass, species, |t| {
                            swarm.grass_rates.get(t).copied().unwrap_or(0)
                        }),
                    );
                }
            }
            for zone in wild
                .zones
                .iter()
                .filter(|z| surface.zones.contains(z.id.as_str()))
            {
                add(
                    land,
                    table_times(&zone.grass, species, |t| {
                        zone.grass_rates.get(t).copied().unwrap_or(0)
                    }),
                );
            }
            if surface.water
                && let Some(table) = &wild.water
            {
                add(
                    "SURF",
                    table_times(table, species, |_| wild.water_rate.unwrap_or(0)),
                );
            }
        }
        if let Some(field) = data.field_encounters.get(map_id) {
            for (method, table) in &field.tables {
                let label = match method.as_str() {
                    "headbutt" if surface.tree => "HEADBUTT",
                    "rock_smash"
                        if map
                            .objects
                            .iter()
                            .any(|o| o.spritemovedata == "SPRITEMOVEDATA_SMASHABLE_ROCK") =>
                    {
                        "ROCK SMASH"
                    }
                    _ => continue,
                };
                if table
                    .common
                    .iter()
                    .chain(&table.rare)
                    .any(|entry| entry.species == species)
                {
                    add(label, ALL);
                }
            }
        }
        if surface.water {
            for (index, group) in data.fishing_encounter_groups_for_map(map_id).enumerate() {
                if group.bite_threshold == 0 {
                    continue;
                }
                for &rod in FISHING_RODS {
                    let Some(table) = group.rod_tables.get(rod) else {
                        continue;
                    };
                    let times = fishing_times(&data.fishing, table, species);
                    if times != 0 {
                        let mut label = rod.replace('_', " ");
                        if index > 0 {
                            label.push_str(" SWARM");
                        }
                        add(&label, times);
                    }
                }
            }
        }
    }
    Ok(rows
        .into_iter()
        .map(|((name, x, y, method), times)| Area {
            name,
            x,
            y,
            method,
            time: time_label(times),
        })
        .collect())
}
