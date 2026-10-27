//! Publish all authored chunks atomically, retaining the original new-game flow.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::encounters::{WildEncounterData, WildEncounterTable};

use crate::authored_route_design::{Interior, RouteDesign};
use crate::authored_route_events as events;
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, CompiledMapExtension, PokegearLandmark, RuntimeMapMetadata, RuntimeSpawnPoint,
};

/// Publishes explicit route strokes and events; it never chooses terrain.
pub(crate) fn build_authored_route(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
    expected_route: &str,
) -> Result<CompiledGamePack> {
    let design = RouteDesign::parse(dossier_json)?;
    ensure!(design.id == expected_route, "wrong route dossier");
    let spec = match expected_route {
        "MN_R01" => RouteSpec {
            group_id: 46,
            group: "MINNESOTA_DRIFTLESS",
            manifest: "minnesota-driftless",
            landmark: "LANDMARK_MN_DRIFTLESS",
            label: "MN_DRIFTLESS",
            name: "DRIFTLESS TRAIL",
            position: [136, 128],
            tileset: "johto",
            music_source: "Route29",
        },
        "MN_R02" => RouteSpec {
            group_id: 47,
            group: "MINNESOTA_PRAIRIE_FARM",
            manifest: "minnesota-prairie-farm",
            landmark: "LANDMARK_MN_PRAIRIE_FARM",
            label: "MN_PRAIRIE_FARM",
            name: "PRAIRIE FARM ROAD",
            position: [112, 128],
            tileset: "kanto",
            music_source: "Route34",
        },
        "MN_R03" => RouteSpec {
            group_id: 48,
            group: "MINNESOTA_RIVER_VALLEY",
            manifest: "minnesota-river-valley",
            landmark: "LANDMARK_MN_RIVER_VALLEY",
            label: "MN_RIVER_VALLEY",
            name: "MINNESOTA VALLEY",
            position: [104, 112],
            tileset: "mn_headwaters",
            music_source: "Route32",
        },
        "MN_R04" => RouteSpec {
            group_id: 49,
            group: "MINNESOTA_TWIN_CITIES",
            manifest: "minnesota-twin-cities",
            landmark: "LANDMARK_MN_TWIN_CITIES",
            label: "MN_TWIN_CITIES",
            name: "TWIN CITIES PARK",
            position: [112, 96],
            tileset: "mn_river_park",
            music_source: "NationalPark",
        },
        "MN_R05" => RouteSpec {
            group_id: 50,
            group: "MINNESOTA_RIVER_BLUFFS",
            manifest: "minnesota-river-bluffs",
            landmark: "LANDMARK_MN_RIVER_BLUFFS",
            label: "MN_RIVER_BLUFFS",
            name: "RIVER BLUFFS",
            position: [96, 80],
            tileset: "mn_headwaters",
            music_source: "Route45",
        },
        "MN_R06" => RouteSpec {
            group_id: 51,
            group: "MINNESOTA_PINE_COUNTRY",
            manifest: "minnesota-pine-country",
            landmark: "LANDMARK_MN_PINE_COUNTRY",
            label: "MN_PINE_COUNTRY",
            name: "PINE COUNTRY ROAD",
            position: [96, 64],
            tileset: "mn_headwaters",
            music_source: "Route37",
        },
        "MN_R07" => RouteSpec {
            group_id: 52,
            group: "MINNESOTA_LEECH_LAKE",
            manifest: "minnesota-leech-lake",
            landmark: "LANDMARK_MN_LEECH_LAKE",
            label: "MN_LEECH_LAKE",
            name: "LEECH LAKE APPROACH",
            position: [88, 48],
            tileset: "mn_headwaters",
            music_source: "Route35",
        },
        "MN_R08" => RouteSpec {
            group_id: 53,
            group: "MINNESOTA_NORTHERN_LAKES",
            manifest: "minnesota-northern-lakes",
            landmark: "LANDMARK_MN_NORTHERN_LAKES",
            label: "MN_NORTHERN_LAKES",
            name: "NORTHERN LAKE CIRCUIT",
            position: [88, 32],
            tileset: "mn_headwaters",
            music_source: "Route42",
        },
        "MN_R09" => RouteSpec {
            group_id: 54,
            group: "MINNESOTA_IRON_RANGE",
            manifest: "minnesota-iron-range",
            landmark: "LANDMARK_MN_IRON_RANGE",
            label: "MN_IRON_RANGE",
            name: "IRON RANGE WEST",
            position: [120, 32],
            tileset: "mn_iron_range",
            music_source: "Route45",
        },
        "MN_R10" => RouteSpec {
            group_id: 55,
            group: "MINNESOTA_IRON_RANGE_DESCENT",
            manifest: "minnesota-iron-range-descent",
            landmark: "LANDMARK_MN_IRON_RANGE_DESCENT",
            label: "MN_IRON_RANGE_DESCENT",
            name: "IRON RANGE DESCENT",
            position: [128, 48],
            tileset: "johto",
            music_source: "Route45",
        },
        "MN_R16" => RouteSpec {
            group_id: 56,
            group: "MINNESOTA_RED_RIVER_PRAIRIE",
            manifest: "minnesota-red-river-prairie",
            landmark: "LANDMARK_MN_RED_RIVER_PRAIRIE",
            label: "MN_RED_RIVER_PRAIRIE",
            name: "RED RIVER PRAIRIE",
            position: [48, 64],
            tileset: "mn_headwaters",
            music_source: "Route34",
        },
        "MN_R17" => RouteSpec {
            group_id: 57,
            group: "MINNESOTA_BORDER_PINES",
            manifest: "minnesota-border-pines",
            landmark: "LANDMARK_MN_BORDER_PINES",
            label: "MN_BORDER_PINES",
            name: "BORDER PINES",
            position: [80, 16],
            tileset: "mn_border_pines",
            music_source: "IlexForest",
        },
        "MN_R18" => RouteSpec {
            group_id: 58,
            group: "MINNESOTA_MILLE_LACS_SHORE",
            manifest: "minnesota-mille-lacs-shore",
            landmark: "LANDMARK_MN_MILLE_LACS_SHORE",
            label: "MN_MILLE_LACS_SHORE",
            name: "MILLE LACS SHORE",
            position: [88, 88],
            tileset: "mn_headwaters",
            music_source: "Route35",
        },
        "MN_R19" => RouteSpec {
            group_id: 59,
            group: "MINNESOTA_PRAIRIE_FOREST_LINK",
            manifest: "minnesota-prairie-forest-link",
            landmark: "LANDMARK_MN_PRAIRIE_FOREST_LINK",
            label: "MN_PRAIRIE_FOREST_LINK",
            name: "PRAIRIE FOREST LINK",
            position: [64, 80],
            tileset: "johto",
            music_source: "Route29",
        },
        "MN_R20" => RouteSpec {
            group_id: 60,
            group: "MINNESOTA_VOYAGEURS_BORDER_LOOP",
            manifest: "minnesota-voyageurs-border-loop",
            landmark: "LANDMARK_MN_VOYAGEURS_BORDER_LOOP",
            label: "MN_VOYAGEURS_BORDER_LOOP",
            name: "VOYAGEURS BORDER LOOP",
            position: [104, 8],
            tileset: "mn_headwaters",
            music_source: "Route42",
        },
        "MN_R11" => RouteSpec {
            group_id: 41,
            group: "MINNESOTA_NORTH_SHORE",
            manifest: "minnesota-north-shore",
            landmark: "LANDMARK_MN_NORTH_SHORE",
            label: "MN_NORTH_SHORE",
            name: "NORTH SHORE TRAIL",
            position: [144, 24],
            tileset: "mn_north_shore",
            music_source: "Route32",
        },
        "MN_R12" => RouteSpec {
            group_id: 42,
            group: "MINNESOTA_GUNFLINT",
            manifest: "minnesota-gunflint",
            landmark: "LANDMARK_MN_GUNFLINT",
            label: "MN_GUNFLINT",
            name: "GUNFLINT TRAIL",
            position: [140, 16],
            tileset: "mn_gunflint",
            music_source: "IlexForest",
        },
        "MN_R13" => RouteSpec {
            group_id: 43,
            group: "MINNESOTA_LEAGUE_APPROACH",
            manifest: "minnesota-league-approach",
            landmark: "LANDMARK_MN_LEAGUE_APPROACH",
            label: "MN_LEAGUE_APPROACH",
            name: "LEAGUE APPROACH",
            position: [120, 24],
            tileset: "johto",
            music_source: "Route45",
        },
        "MN_R14" => RouteSpec {
            group_id: 44,
            group: "MINNESOTA_HEADWATERS",
            manifest: "minnesota-headwaters",
            landmark: "LANDMARK_MN_HEADWATERS",
            label: "MN_HEADWATERS",
            name: "HEADWATERS TRAIL",
            position: [80, 40],
            tileset: "mn_headwaters",
            music_source: "Route30",
        },
        "MN_R15" => RouteSpec {
            group_id: 45,
            group: "MINNESOTA_CENTRAL_LAKES",
            manifest: "minnesota-central-lakes",
            landmark: "LANDMARK_MN_CENTRAL_LAKES",
            label: "MN_CENTRAL_LAKES",
            name: "CENTRAL LAKES WEST",
            position: [64, 72],
            tileset: "kanto",
            music_source: "Route12",
        },
        _ => anyhow::bail!("unsupported authored route"),
    };
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .data
            .maps
            .contains_key(&events::chunk(&design.map_prefix, [0, 0])),
        "authored route is already installed"
    );
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|map| map.group_id == spec.group_id),
        "authored route map group {} is already occupied",
        spec.group_id
    );
    let layout = Layout::parse(layout_json)?;
    ensure!(layout.id == design.id, "layout and dossier differ");
    let blocks = layout.compile()?;
    let mut pack = if base.data.tilesets.contains_key(spec.tileset) {
        base.clone()
    } else if expected_route == "MN_R04" {
        crate::twin_cities_art::build_twin_cities_art_pack(base)?
    } else if expected_route == "MN_R09" {
        crate::build_iron_range_art_pack(base)?
    } else if expected_route == "MN_R17" {
        crate::build_border_pines_art_pack(base)?
    } else if expected_route == "MN_R11" {
        crate::build_north_shore_art_pack(base)?
    } else if matches!(
        expected_route,
        "MN_R03"
            | "MN_R05"
            | "MN_R06"
            | "MN_R07"
            | "MN_R08"
            | "MN_R14"
            | "MN_R16"
            | "MN_R18"
            | "MN_R20"
    ) {
        crate::headwaters_art::build_headwaters_art_pack(base)?
    } else {
        crate::build_gunflint_art_pack(base)?
    };
    // Older supplied packs retain the exact Cut definition but omit its
    // executable root. Recompile that existing catalog; never synthesize scripts.
    pack.data
        .materialize_global_scripts()
        .context("materialize native field script roots")?;
    let reachable = validate_geometry(&pack, &design, &blocks, spec.tileset)?;
    let mut modules = Vec::new();
    for row in 0..design.chunk_rows {
        for col in 0..design.chunk_columns {
            let name = events::chunk(&design.map_prefix, [col * 64, row * 48]);
            let mut module = authoring::blank(&pack, "Route32", &name, &events::constant(&name))?;
            module.attributes.width = (design.dimensions_metatiles[0] - col * 32).min(32);
            module.attributes.height = (design.dimensions_metatiles[1] - row * 24).min(24);
            module.attributes.tileset_name = layout.compiled_tileset.clone();
            module.attributes.border_block = match spec.tileset {
                "kanto" => 0x0f,
                "mn_river_park" => 0,
                _ => 5,
            };
            module.attributes.music = pack
                .data
                .maps
                .get(spec.music_source)
                .context("authored music source")?
                .attributes
                .music
                .clone();
            module.blocks.clear();
            for y in row * 24..row * 24 + module.attributes.height {
                module.blocks.extend_from_slice(
                    &blocks[usize::from(y) * usize::from(design.dimensions_metatiles[0])
                        + usize::from(col) * 32
                        ..usize::from(y) * usize::from(design.dimensions_metatiles[0])
                            + usize::from(col) * 32
                            + usize::from(module.attributes.width)],
                );
            }
            for (direction, dx, dy) in [
                ("north", 0, -1),
                ("south", 0, 1),
                ("west", -1, 0),
                ("east", 1, 0),
            ] {
                let [x, y] = [i32::from(col) + dx, i32::from(row) + dy];
                if x >= 0
                    && x < i32::from(design.chunk_columns)
                    && y >= 0
                    && y < i32::from(design.chunk_rows)
                {
                    module.attributes.connections.push(MapConnection {
                        direction: direction.into(),
                        target_map: events::chunk(
                            &design.map_prefix,
                            [u16::try_from(x)? * 64, u16::try_from(y)? * 48],
                        ),
                        offset: 0,
                    });
                }
            }
            module.attributes.connection_flags =
                (!module.attributes.connections.is_empty()).then(|| {
                    module
                        .attributes
                        .connections
                        .iter()
                        .map(|c| c.direction.to_ascii_uppercase())
                        .collect::<Vec<_>>()
                        .join(" | ")
                });
            modules.push(module);
        }
    }
    for door in &design.doors {
        let room = design
            .interiors
            .iter()
            .find(|room| room.id == door.target)
            .context("door interior")?;
        let passive_destination = room
            .passive_arrival
            .as_ref()
            .is_some_and(|arrival| door.target_warp == i16::try_from(arrival.index).unwrap_or(-1));
        ensure!(
            (door.arrival_tile == door.tile || door.return_warp_index.is_some())
                && door.target_warp > 0
                && (passive_destination
                    || usize::try_from(door.target_warp).is_ok_and(|id| id <= room.exits.len()))
                && !room.exits.is_empty(),
            "retain native reciprocal door identities"
        );
        if room.exit_doors.is_empty() {
            ensure!(
                door.target_warp == 1 || passive_destination,
                "single-mouth room must enter its first warp or declared passive floor"
            );
        } else {
            ensure!(
                room.exit_doors.len() == room.exits.len()
                    && room.exit_doors.iter().collect::<BTreeSet<_>>().len()
                        == room.exit_doors.len()
                    && room.exits.iter().collect::<BTreeSet<_>>().len() == room.exits.len()
                    && room.exit_doors.get(usize::try_from(door.target_warp - 1)?)
                        == Some(&door.tile),
                "rock-cut entrance must match its own interior warp"
            );
        }
        let exterior = modules
            .iter_mut()
            .find(|map| map.id == events::chunk(&design.map_prefix, door.tile))
            .context("exterior chunk")?;
        ensure!(
            door.warp_index == u16::try_from(exterior.events.warps.len() + 1)?,
            "exterior warp order differs from design"
        );
        let [x, y] = events::local(door.tile);
        authoring::warp(
            exterior,
            (x, y),
            &events::constant(&room.id),
            door.target_warp,
        );
        if let Some(index) = door.return_warp_index {
            ensure!(
                usize::from(index) == exterior.events.warps.len() + 1
                    && door.arrival_tile == door.return_tile
                    && collision(
                        &pack,
                        &blocks,
                        design.dimensions_metatiles[0],
                        door.arrival_tile,
                        spec.tileset
                    )? == "FLOOR",
                "return-only warp must target the authored safe floor"
            );
            // FLOOR does not trigger a warp during movement. This ordinary
            // native event is only a destination for the interior's return.
            let [x, y] = events::local(door.arrival_tile);
            authoring::warp(exterior, (x, y), &events::constant(&room.id), 1);
        }
    }
    events::apply(&mut pack, &design, &mut modules)?;
    for room in &design.interiors {
        let door = design
            .doors
            .iter()
            .find(|door| door.target == room.id)
            .context("room exterior door")?;
        let mut interior =
            authoring::blank(&pack, &room.template, &room.id, &events::constant(&room.id))?;
        authoring::blocks(
            &mut interior,
            room.dimensions_metatiles[0],
            room.dimensions_metatiles[1],
            &room.rows.join(" "),
        )?;
        if let Some(border) = room.border_block {
            interior.attributes.border_block = border;
        }
        validate_room(&pack, &interior, room)?;
        for (index, &[x, y]) in room.exits.iter().enumerate() {
            let door = if room.exit_doors.is_empty() {
                door
            } else {
                design
                    .doors
                    .iter()
                    .find(|door| {
                        door.target == room.id && room.exit_doors.get(index) == Some(&door.tile)
                    })
                    .context("interior exit has no corresponding exterior mouth")?
            };
            authoring::warp(
                &mut interior,
                (x, y),
                &events::constant(&events::chunk(&design.map_prefix, door.tile)),
                i16::try_from(door.return_warp_index.unwrap_or(door.warp_index))?,
            );
        }
        if let Some(arrival) = &room.passive_arrival {
            // FLOOR never activates this record when stepped on. The record
            // supplies a safe destination for the exterior entrance only.
            let [x, y] = arrival.tile;
            authoring::warp(
                &mut interior,
                (x, y),
                &events::constant(&events::chunk(&design.map_prefix, door.tile)),
                i16::try_from(door.return_warp_index.unwrap_or(door.warp_index))?,
            );
        }
        events::room_events(&pack, &mut interior, room)?;
        modules.push(interior);
    }
    register_constants(&mut pack, &design, &spec)?;
    if expected_route == "MN_R18" {
        crate::mille_lacs_shore_modpack::bind_shore_watch_quest(
            &mut pack,
            &mut modules,
            dossier_json,
        )?;
    }
    if expected_route == "MN_R19" {
        crate::prairie_forest_link_modpack::bind_prairie_forest_link(
            &mut pack,
            &mut modules,
            dossier_json,
        )?;
    }
    let landmark = spec.landmark;
    ensure!(
        !pack
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|entry| entry.constant == landmark),
        "authored route landmark already exists"
    );
    let landmark_id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("source landmarks")?
        + 1;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark_id,
            constant: landmark.into(),
            label: spec.label.into(),
            name: spec.name.into(),
            x: spec.position[0],
            y: spec.position[1],
            region: "JOHTO".into(),
        });
    let temporary_spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("source spawns")?
        .checked_add(1)
        .context("temporary spawn identifier overflow")?;
    for (index, mut module) in modules.into_iter().enumerate() {
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{}", spec.group));
        module.attributes.location = Some(landmark.into());
        let name = module.id.clone();
        let constant = events::constant(&name);
        let room = design.interiors.iter().find(|room| room.id == name);
        let point = if let Some(room) = room {
            room.clear_arrival.unwrap_or([4, 7])
        } else {
            events::local(
                *reachable
                    .iter()
                    .find(|&&tile| {
                        events::chunk(&design.map_prefix, tile) == name
                            && collision(
                                &pack,
                                &blocks,
                                design.dimensions_metatiles[0],
                                tile,
                                spec.tileset,
                            )
                            .is_ok_and(|c| c == "FLOOR")
                    })
                    .with_context(|| format!("no connected floor in {name}"))?,
            )
        };
        let map_id = u16::try_from(index + 1)?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), landmark.into());
        crate::insert_compiled_map_extension(
            &mut pack,
            CompiledMapExtension {
                manifest_id: format!("{}-{index}", spec.manifest),
                map_name: name.clone(),
                map_constant: constant.clone(),
                metadata: RuntimeMapMetadata {
                    constant: constant.clone(),
                    name: name.clone(),
                    group_name: spec.group.into(),
                    group_id: spec.group_id,
                    map_id,
                    width: module.attributes.width,
                    height: module.attributes.height,
                    environment: module
                        .attributes
                        .environment
                        .clone()
                        .context("map environment")?,
                    phone_service: module.attributes.phone_service,
                },
                spawn_key: temporary_spawn.to_string(),
                spawn: RuntimeSpawnPoint {
                    identifier: temporary_spawn,
                    map_constant: constant,
                    map_name: name.clone(),
                    group_id: i16::try_from(spec.group_id)?,
                    map_id: i16::try_from(map_id)?,
                    tile_x: i16::try_from(point[0])?,
                    tile_y: i16::try_from(point[1])?,
                    group_name: spec.group.into(),
                    metatile_x: i16::try_from(point[0] / 2)?,
                    metatile_y: i16::try_from(point[1] / 2)?,
                    subtile_x: i16::try_from(point[0] % 2)?,
                    subtile_y: i16::try_from(point[1] % 2)?,
                },
                module,
                wild_encounters: if room.is_some() {
                    None
                } else {
                    Some(wild(&design, &name))
                },
                start_new_game_here: false,
            },
        )?;
        // These entries validate each map extension but are not new recovery
        // destinations. Retain every original spawn and the original HOME.
        pack.data
            .runtime_spawn_points
            .remove(&temporary_spawn.to_string());
    }
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
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

fn register_constants(
    pack: &mut CompiledGamePack,
    design: &RouteDesign,
    spec: &RouteSpec,
) -> Result<()> {
    let global = &mut pack.data.story_event_script_constants.global;
    let group_constant = format!("MAPGROUP_{}", spec.group);
    ensure!(
        !global.contains_key(&group_constant),
        "authored map group constant already exists"
    );
    global.insert(group_constant, i64::from(spec.group_id));
    let next = global
        .iter()
        .filter(|(k, _)| k.starts_with("EVENT_"))
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("event flag identifier overflow")?;
    for (index, flag) in design
        .trainers
        .iter()
        .map(|t| t.defeated_flag.as_str())
        .chain(design.items.iter().map(|i| i.flag.as_str()))
        .chain(
            design
                .interiors
                .iter()
                .flat_map(|room| &room.items)
                .map(|item| item.flag.as_str()),
        )
        .enumerate()
    {
        ensure!(
            flag.starts_with(&format!("EVENT_{}_", design.id)) && !global.contains_key(flag),
            "event flag collision {flag}"
        );
        let value = next
            .checked_add(i64::try_from(index)?)
            .context("event flag identifier overflow")?;
        global.insert(flag.into(), value);
    }
    for (index, trainer) in design.trainers.iter().enumerate() {
        ensure!(
            !global.contains_key(&trainer.id),
            "trainer constant collision {}",
            trainer.id
        );
        global.insert(trainer.id.clone(), i64::try_from(index + 1)?);
    }
    Ok(())
}

fn wild(design: &RouteDesign, name: &str) -> WildEncounterData {
    let table = &design.encounters;
    let water = WildEncounterTable {
        morning: table.water.clone(),
        day: table.water.clone(),
        night: table.water.clone(),
    };
    WildEncounterData {
        map_name: name.into(),
        grass_rates: Some(table.grass_rates.clone()),
        grass: Some(table.grass.clone()),
        water_rate: Some(table.water_rate),
        water: Some(water),
        ..Default::default()
    }
}

pub(super) fn collision<'a>(
    pack: &'a CompiledGamePack,
    blocks: &[u16],
    width: u16,
    [x, y]: [u16; 2],
    tileset: &str,
) -> Result<&'a str> {
    ensure!(
        x < width * 2 && usize::from(y / 2) < blocks.len() / usize::from(width),
        "anchor outside authored blocks"
    );
    let block = blocks[usize::from(y / 2) * usize::from(width) + usize::from(x / 2)];
    let quadrants = pack
        .data
        .tilesets
        .get(tileset)
        .context("source tileset")?
        .collision
        .get(&format!("{block:02x}"))
        .context("source collision block")?;
    Ok(quadrants
        .get(usize::from(y % 2) * 2 + usize::from(x % 2))
        .context("source collision quadrant")?)
}

pub(crate) fn validate_geometry(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
) -> Result<BTreeSet<[u16; 2]>> {
    let bodies = design
        .trainers
        .iter()
        .map(|t| t.tile)
        .chain(design.residents.iter().map(|n| n.tile))
        .chain(design.items.iter().map(|i| i.tile))
        .collect::<Vec<_>>();
    let occupied = bodies.iter().copied().collect::<BTreeSet<_>>();
    ensure!(occupied.len() == bodies.len(), "overlapping route actors");
    let [width, height] = design.dimensions_metatiles;
    let kind = |tile| collision(pack, blocks, width, tile, tileset);
    crate::authored_route_traversal::validate_ledges(design, kind)?;
    for tile in bodies {
        let [x, y] = events::local(tile);
        ensure!(
            x > 0 && x < 63 && y > 0 && y < 47,
            "actor {tile:?} touches a chunk boundary; native talk cannot cross that boundary"
        );
        ensure!(
            kind(tile)? == "FLOOR",
            "route actor {tile:?} is not on clear floor"
        );
    }
    let start = design.endpoints[0].gate_metatile.map(|value| value * 2);
    let end = design.endpoints[1].gate_metatile.map(|value| value * 2);
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    ensure!(kind(start)? == "FLOOR", "route arrival is blocked");
    while let Some([x, y]) = queue.pop_front() {
        for [nx, ny] in [
            [i32::from(x) + 1, i32::from(y)],
            [i32::from(x) - 1, i32::from(y)],
            [i32::from(x), i32::from(y) + 1],
            [i32::from(x), i32::from(y) - 1],
        ] {
            if !(0..i32::from(width) * 2).contains(&nx) || !(0..i32::from(height) * 2).contains(&ny)
            {
                continue;
            }
            let tile = [u16::try_from(nx)?, u16::try_from(ny)?];
            if occupied.contains(&tile) || seen.contains(&tile) {
                continue;
            }
            let terrain = kind(tile)?;
            if matches!(terrain, "FLOOR" | "LADDER" | "DOOR")
                || ((!design.required_grass_crossings.is_empty()
                    || !design.battle_or_grass_choices.is_empty())
                    && terrain == "TALL_GRASS")
            {
                seen.insert(tile);
                queue.push_back(tile);
            }
        }
    }
    ensure!(seen.contains(&end), "route ends disconnected");
    for crossing in &design.required_grass_crossings {
        let [x0, y0, x1, y1] = crossing.bounds_core;
        ensure!(
            x0 < x1 && y0 < y1 && x1 <= width * 2 && y1 <= height * 2,
            "{}: invalid grass crossing bounds",
            crossing.id
        );
        for y in y0..y1 {
            for x in x0..x1 {
                ensure!(
                    kind([x, y])? == "TALL_GRASS",
                    "{}: required grass crossing contains another terrain",
                    crossing.id
                );
            }
        }
    }
    let cut = crate::authored_route_cut_gates::validate(pack, design, blocks, tileset, &seen)?;
    let all_seen = cut.as_ref().map_or(&seen, |cut| &cut.after);
    let after_blocks = cut
        .as_ref()
        .map_or(blocks, |cut| cut.after_blocks.as_slice());
    let after_kind = |tile| collision(pack, after_blocks, width, tile, tileset);
    crate::authored_route_traversal::validate_grass_crossings(design, &seen)?;
    crate::authored_route_traversal::validate_grass_crossings(design, all_seen)?;
    crate::authored_route_traversal::validate_ledge_returns(design, all_seen, after_kind)?;
    crate::authored_route_traversal::validate_battle_grass_choices(design, &seen, kind)?;
    crate::authored_route_traversal::validate_battle_grass_choices(design, all_seen, after_kind)?;
    let surf_only = design
        .surf_only_actor_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    ensure!(
        surf_only.len() == design.surf_only_actor_ids.len()
            && surf_only
                .iter()
                .all(|id| design.items.iter().any(|item| item.id == *id)),
        "Surf-only actors must be unique declared pickups"
    );
    let surf_reachable = if surf_only.is_empty() {
        BTreeSet::new()
    } else {
        crate::authored_route_surface::reachable_with_surf(
            pack,
            design,
            after_blocks,
            tileset,
            &occupied,
        )?
    };
    for (id, tile) in design
        .trainers
        .iter()
        .map(|a| (a.id.as_str(), a.tile))
        .chain(design.residents.iter().map(|a| (a.id.as_str(), a.tile)))
        .chain(design.items.iter().map(|a| (a.id.as_str(), a.tile)))
    {
        let has_approach = |region: &BTreeSet<[u16; 2]>| {
            [
                [tile[0].saturating_sub(1), tile[1]],
                [tile[0] + 1, tile[1]],
                [tile[0], tile[1].saturating_sub(1)],
                [tile[0], tile[1] + 1],
            ]
            .iter()
            .any(|point| {
                region.contains(point)
                    && events::chunk(&design.map_prefix, *point)
                        == events::chunk(&design.map_prefix, tile)
            })
        };
        let actor_seen = if surf_only.contains(id) {
            ensure!(
                !has_approach(&seen) && !has_approach(all_seen),
                "Surf-only pickup {id} has an ordinary walking approach"
            );
            &surf_reachable
        } else if cut
            .as_ref()
            .is_some_and(|cut| cut.gated_actor_ids.contains(id))
        {
            all_seen
        } else {
            &seen
        };
        ensure!(
            [
                [tile[0].saturating_sub(1), tile[1]],
                [tile[0] + 1, tile[1]],
                [tile[0], tile[1].saturating_sub(1)],
                [tile[0], tile[1] + 1]
            ]
            .iter()
            .any(|p| actor_seen.contains(p) && events::chunk(&design.map_prefix,*p) == events::chunk(&design.map_prefix,tile)),
            "actor {tile:?} has no clear approach"
        );
    }
    for sign in &design.signs {
        ensure!(
            kind(sign.tile)? == "WALL",
            "sign must match its visible source block"
        );
    }
    for door in &design.doors {
        ensure!(
            kind(door.tile)? == door.collision && seen.contains(&door.return_tile),
            "authored door or return is blocked"
        );
    }
    for viewpoint in &design.viewpoints {
        ensure!(all_seen.contains(&viewpoint.tile), "unreachable viewpoint");
    }
    if let Some(shortcut) = &design.shortcut {
        ensure!(
            kind(shortcut.tile)? == "CUT_TREE",
            "missing native Cut tree"
        );
        ensure!(seen.contains(&shortcut.approach), "Cut approach is blocked");
    }
    crate::authored_route_cut_gates::validate_shortcut(pack, design, blocks, tileset, &seen)?;
    crate::authored_route_surface::validate_outer_boundary(
        pack, design, blocks, tileset, &occupied,
    )
    .context("Surf edge safety")?;
    Ok(seen)
}

struct RouteSpec {
    group_id: u16,
    group: &'static str,
    manifest: &'static str,
    landmark: &'static str,
    label: &'static str,
    name: &'static str,
    position: [i16; 2],
    tileset: &'static str,
    music_source: &'static str,
}

pub(crate) fn validate_room(
    pack: &CompiledGamePack,
    module: &crate::MapModule,
    room: &Interior,
) -> Result<()> {
    let kind = |tile| {
        collision(
            pack,
            &module.blocks,
            module.attributes.width,
            tile,
            &module.attributes.tileset_name,
        )
    };
    for &tile in &room.exits {
        ensure!(
            kind(tile)? == room.exit_collision,
            "room exit collision differs from authored recipe"
        );
    }
    let bodies = room
        .residents
        .iter()
        .map(|resident| resident.tile)
        .chain(room.items.iter().map(|item| item.tile))
        .chain(room.keeper_tile)
        .collect::<BTreeSet<_>>();
    if let Some(arrival) = &room.passive_arrival {
        ensure!(
            room.exit_doors.is_empty()
                && usize::from(arrival.index) == room.exits.len() + 1
                && kind(arrival.tile)? == "FLOOR"
                && !bodies.contains(&arrival.tile)
                && !room.exits.contains(&arrival.tile)
                && room.clear_arrival == Some(arrival.tile),
            "passive room arrival must be the next native record on clear unoccupied floor"
        );
    }
    for &tile in &bodies {
        ensure!(kind(tile)? == "FLOOR", "room actor stands on furniture");
    }
    let start = room.clear_arrival.unwrap_or([4, 7]);
    ensure!(
        kind(start)? == "FLOOR" && !bodies.contains(&start),
        "room arrival is blocked"
    );
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for [nx, ny] in [
            [i32::from(x) - 1, i32::from(y)],
            [i32::from(x) + 1, i32::from(y)],
            [i32::from(x), i32::from(y) - 1],
            [i32::from(x), i32::from(y) + 1],
        ] {
            if !(0..i32::from(module.attributes.width) * 2).contains(&nx)
                || !(0..i32::from(module.attributes.height) * 2).contains(&ny)
            {
                continue;
            }
            let tile = [u16::try_from(nx)?, u16::try_from(ny)?];
            if seen.contains(&tile) || bodies.contains(&tile) {
                continue;
            }
            if matches!(kind(tile)?, "FLOOR" | "LADDER" | "WARP_CARPET_DOWN") {
                seen.insert(tile);
                queue.push_back(tile);
            }
        }
    }
    for tile in bodies
        .into_iter()
        .chain(room.signs.iter().map(|sign| sign.tile))
        .chain(room.exits.iter().copied())
    {
        ensure!(
            [
                [tile[0].saturating_sub(1), tile[1]],
                [tile[0] + 1, tile[1]],
                [tile[0], tile[1].saturating_sub(1)],
                [tile[0], tile[1] + 1]
            ]
            .iter()
            .any(|tile| seen.contains(tile)),
            "room event has no reachable approach"
        );
    }
    for sign in &room.signs {
        ensure!(kind(sign.tile)? == "WALL", "room sign is off its wall");
    }
    Ok(())
}
