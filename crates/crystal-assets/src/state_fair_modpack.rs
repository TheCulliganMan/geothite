//! A native-tile fair with independent NPC vendors and ordinary counter input.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::encounters::WildEncounterData;
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_city_expansion_events;
use crate::authored_route_design::RouteTrainer;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;

#[derive(Deserialize)]
struct FairDesign {
    native_map: Town,
    native_flags: Vec<String>,
    native_trainers: Vec<RouteTrainer>,
    native_city_connection: CityConnection,
    native_room_edits: Vec<RoomEdit>,
    native_encounters: WildEncounterData,
    native_encounter_families: Vec<crate::state_fair_encounters::Family>,
    native_patrols: Vec<crate::state_fair_patrols::Patrol>,
    #[serde(default)]
    native_ambient: Vec<crate::state_fair_patrols::Patrol>,
}

#[derive(Deserialize)]
struct RoomEdit {
    map: String,
    position: [usize; 2],
    before: String,
    after: String,
}

#[derive(Deserialize)]
struct CityConnection {
    map: String,
    tileset: String,
    #[serde(default)]
    dimensions_metatiles: Option<[u16; 2]>,
    #[serde(default)]
    fair_to_city_offset: i32,
    edits: Vec<CityEdit>,
}

#[derive(Deserialize)]
struct CityEdit {
    position: [usize; 2],
    before: String,
    after: String,
}

/// Add the authored fairgrounds, native source rooms and separate counter NPCs.
///
/// The parent must already contain the selected regional food and carnival
/// species. Bounded native grass habitats make every selected evolution available.
///
/// A two-core-wide street and reciprocal connection bind St Paul's north edge
/// to the fair. Only the declared city border/grass blocks and that
/// connection change. Other maps, HOME, runtime files and audio are retained.
/// No badges, plot progress or healing are granted by this publication.
///
/// # Errors
/// Returns an error for stale identities, missing native art/templates, occupied
/// flags/items, missing regional species, invalid counter/door or habitat geometry,
/// or final pack verification failure.
pub fn build_state_fair_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    design_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: FairDesign = serde_json::from_str(design_json)?;
    let town = &source.native_map;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnStateFairgrounds"
            && town.native_constant == "MN_STATE_FAIRGROUNDS"
            && town.compiled_tileset == "mn_state_fair_native"
            && town.dimensions_metatiles == [48, 36]
            && layout.id == town.id
            && layout.compiled_tileset == town.compiled_tileset
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && !base.data.maps.contains_key(&town.id),
        "native fair identity or extent differs"
    );
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        layout_json,
        "mn_state_fair_native",
        "johto",
        "state-fair-native",
    )?;
    authored_city_expansion_events::flags(&mut pack, &source.native_flags)?;
    install_food(&mut pack)?;
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .chain(std::iter::once((
            town.native_constant.clone(),
            town.id.clone(),
        )))
        .chain(
            town.interiors
                .iter()
                .map(|r| (r.native_constant.clone(), r.id.clone())),
        )
        .collect::<BTreeMap<_, _>>();
    let mut exterior = authoring::blank(&pack, "NewBarkTown", &town.id, &town.native_constant)?;
    exterior.attributes.width = town.dimensions_metatiles[0];
    exterior.attributes.height = town.dimensions_metatiles[1];
    exterior.attributes.border_block = 5;
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    exterior.blocks = layout.compile()?;
    authored_city_expansion_events::ordinary(
        &mut pack,
        &mut exterior,
        &town.residents,
        &town.signs,
        "MN_ST_PAUL_POKECENTER_1F",
        "MART_GOLDENROD_2F_1",
    )?;
    authored_city_expansion_events::activities(
        &mut pack,
        &mut exterior,
        &source.native_trainers,
        &[],
    )?;
    crate::state_fair_patrols::install(&pack, &mut exterior, &source.native_patrols)?;
    crate::state_fair_patrols::install(&pack, &mut exterior, &source.native_ambient)?;
    for door in &town.doors {
        ensure!(
            usize::from(door.warp_index) == exterior.events.warps.len() + 1,
            "fair door records must stay in native order"
        );
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("fair room")?;
        authoring::warp(
            &mut exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            door.target_warp,
        );
        authoring::warp(
            &mut exterior,
            (door.return_tile[0], door.return_tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("fair return")?)?,
        );
    }
    let mut modules = vec![(exterior, [16, 68])];
    for room in &town.interiors {
        let mut map = authoring::blank(&pack, &room.template, &room.id, &room.native_constant)?;
        ensure!(
            [map.attributes.width, map.attributes.height] == room.dimensions_metatiles,
            "native room template extent changed"
        );
        map.attributes.tileset_name = room.tileset.clone();
        map.attributes.border_block = room.border_block;
        for edit in source.native_room_edits.iter().filter(|e| e.map == room.id) {
            let [x, y] = edit.position;
            ensure!(
                x < usize::from(map.attributes.width) && y < usize::from(map.attributes.height),
                "fair room edit outside map"
            );
            let block = &mut map.blocks[y * usize::from(map.attributes.width) + x];
            ensure!(
                *block == u16::from_str_radix(&edit.before, 16)?,
                "fair room native source changed"
            );
            *block = u16::from_str_radix(&edit.after, 16)?;
        }
        authored_city_expansion_events::ordinary(
            &mut pack,
            &mut map,
            &room.residents,
            &room.signs,
            "MN_ST_PAUL_POKECENTER_1F",
            "MART_GOLDENROD_2F_1",
        )?;
        for exit in &room.exits {
            authoring::warp(
                &mut map,
                (exit.tile[0], exit.tile[1]),
                &town.native_constant,
                exit.target_warp,
            );
        }
        let return_index = town
            .doors
            .iter()
            .find(|d| d.target == room.id)
            .and_then(|d| d.return_warp_index)
            .context("fair room passive return")?;
        authoring::warp(
            &mut map,
            (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
            &town.native_constant,
            i16::try_from(return_index)?,
        );
        modules.push((map, room.passive_arrival.tile));
    }
    crate::state_fair_scripts::install(&mut modules)?;
    for (map, _) in &mut modules {
        authored_city_expansion_events::finish(&pack, map, &names)?;
    }
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        town,
        &modules,
        &BTreeSet::from([[16, 71], [17, 71]]),
    )?;
    modules[0].0.attributes.connections = vec![MapConnection {
        direction: "south".into(),
        target_map: "MnStPaul".into(),
        offset: source.native_city_connection.fair_to_city_offset,
    }];
    modules[0].0.attributes.connection_flags = Some("SOUTH".into());
    let parent_landmark = pack
        .data
        .pokegear_landmarks
        .map_to_landmark
        .get("MnStPaul")
        .context("installed St Paul landmark")?
        .clone();
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_STATE_FAIR",
            group_id: 80,
            landmark: "LANDMARK_MN_STATE_FAIR",
            fallback_landmark: Some(&parent_landmark),
            constant: "MN_STATE_FAIRGROUNDS",
            name: "STATE FAIR",
            position: [107, 102],
            manifest: "minnesota-state-fair-dense-v1",
            town: "MnStateFairgrounds",
            clinic: "",
            gym: "",
        },
    )?;
    bind_city(&mut pack, &source.native_city_connection)?;
    crate::state_fair_encounters::install(
        &mut pack,
        source.native_encounters,
        &source.native_encounter_families,
    )?;
    pack.report.maps = pack.data.maps.len();
    pack.report.items = pack.data.items.len();
    pack.report
        .manifests
        .push("minnesota-state-fair-grass-v1".into());
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

fn bind_city(pack: &mut CompiledGamePack, connection: &CityConnection) -> Result<()> {
    let dimensions = connection.dimensions_metatiles.unwrap_or([64, 48]);
    let (street_x, street_length, offset) = match dimensions {
        [64, 48] => (8, 8, 0),
        [40, 34] => (5, 5, 3),
        _ => anyhow::bail!("unsupported St Paul fair approach extent"),
    };
    ensure!(
        connection.map == "MnStPaul"
            && connection.tileset == "kanto"
            && connection.edits.len() == street_length
            && connection.fair_to_city_offset == offset,
        "expected the bounded St Paul fair street"
    );
    let mut city = pack
        .data
        .maps
        .get(&connection.map)
        .context("St Paul fair access")?
        .clone();
    ensure!(
        city.attributes.tileset_name == connection.tileset
            && [city.attributes.width, city.attributes.height] == dimensions
            && !city
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == "north"),
        "St Paul extent, native art or existing north connection changed"
    );
    for (y, edit) in connection.edits.iter().enumerate() {
        ensure!(
            edit.position == [street_x, y] && edit.after == "55",
            "fair street must follow the declared clear northern approach"
        );
        let index = y * usize::from(city.attributes.width) + street_x;
        ensure!(
            city.blocks[index] == u16::from_str_radix(&edit.before, 16)?,
            "St Paul fair street source block changed at {:?}",
            edit.position
        );
        city.blocks[index] = 0x55;
    }
    city.attributes.connections.push(MapConnection {
        direction: "north".into(),
        target_map: "MnStateFairgrounds".into(),
        offset: -offset,
    });
    city.attributes.connection_flags = Some(
        city.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    authored_town_publication::replace(pack, city)
}

fn install_food(pack: &mut CompiledGamePack) -> Result<()> {
    for (id, name, price, healing, description) in [
        (
            "FAIR_COOKIE_PAIL",
            "COOKIE PAIL",
            500,
            50,
            "Warm cookies.\nRestores 50 HP.",
        ),
        (
            "FAIR_CORN_STICK",
            "SWEET CORN",
            300,
            20,
            "Roast sweet corn.\nRestores 20 HP.",
        ),
        (
            "FAIR_PRETZEL",
            "PRETZEL",
            200,
            20,
            "Warm pretzel.\nRestores 20 HP.",
        ),
        (
            "FAIR_CHEESE_CURDS",
            "CHEESE CURDS",
            400,
            30,
            "Cheese curds.\nRestores 30 HP.",
        ),
        (
            "FAIR_ROOT_BEER",
            "ROOT BEER",
            500,
            60,
            "Cold root beer.\nRestores 60 HP.",
        ),
        (
            "FAIR_ICE_CREAM",
            "ICE CREAM",
            450,
            50,
            "Dairy ice cream.\nRestores 50 HP.",
        ),
        (
            "FAIR_PICKLE",
            "PICKLE STICK",
            100,
            10,
            "A pickle stick.\nRestores 10 HP.",
        ),
        (
            "FAIR_KETTLE_CORN",
            "KETTLE CORN",
            300,
            30,
            "Kettle corn.\nRestores 30 HP.",
        ),
        (
            "FAIR_HONEY",
            "FAIR HONEY",
            250,
            20,
            "Local honey.\nRestores 20 HP.",
        ),
        (
            "FAIR_HOTDISH",
            "HOTDISH",
            650,
            80,
            "Tater tot hotdish.\nRestores 80 HP.",
        ),
        (
            "FAIR_PRONTO_PUP",
            "PRONTO PUP",
            250,
            30,
            "A pup on a stick.\nRestores 30 HP.",
        ),
        (
            "FAIR_MINI_DONUTS",
            "MINI DONUTS",
            350,
            50,
            "Warm mini donuts.\nRestores 50 HP.",
        ),
        (
            "FAIR_FUNNEL_CAKE",
            "FUNNEL CAKE",
            500,
            60,
            "Sweet funnel cake.\nRestores 60 HP.",
        ),
        (
            "FAIR_FRIED_CANDY",
            "FRIED CANDY",
            600,
            80,
            "Fried candy bar.\nRestores 80 HP.",
        ),
        (
            "FAIR_WALLEYE",
            "WALLEYE",
            800,
            100,
            "Fried walleye.\nRestores 100 HP.",
        ),
        (
            "FAIR_BEER",
            "FAIR BEER",
            500,
            100,
            "Heals 100 HP, but\ncauses confusion.",
        ),
    ] {
        ensure!(
            name.chars().count() <= 12,
            "fair food name exceeds native item row: {id}"
        );
        ensure!(
            description.lines().count() <= 2
                && description.lines().all(|line| line.chars().count() <= 18),
            "fair food description exceeds native Pack textbox: {id}"
        );
        let mut item = pack
            .data
            .items
            .get("POTION")
            .context("native healing item")?
            .clone();
        item.name = name.into();
        item.description = description.into();
        item.price = price;
        item.parameter = healing;
        item.script_name = id.into();
        if id == "FAIR_BEER" {
            item.effect = "RESTORE_HP_CONFUSE".into();
            item.confusion_inflict_turns = Some(4);
        }
        ensure!(
            pack.data.items.insert(id.into(), item).is_none(),
            "occupied fair food {id}"
        );
    }
    Ok(())
}
