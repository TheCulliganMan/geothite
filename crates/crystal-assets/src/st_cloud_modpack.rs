//! St Cloud's native city doors and preserved canonical fifth-gym wind course.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::authored_route_layout::Layout;
use crate::authored_town_publication::{self, CityIdentity};
use crate::canonical_city_gym::{self, Binding};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Publishes St Cloud, its public rooms and the canonical fifth gym.
///
/// The native gym wind lanes, trainers, grille, badge and TM stay intact.
/// Only its first three warp destinations and landmark change. Ordinary care,
/// PC and shopping use production services. Reciprocal public R05/R06 seams are
/// installed and checked; no badge or story progress is granted. The station
/// public room is accessible; rail service remains separate work.
/// Existing HOME, recovery, unrelated maps, source art and audio are preserved.
///
/// # Errors
/// Returns an error for stale sources/gym content, occupied identities, unsafe
/// geometry, invalid native scripts/doors, incorrect edge projections or pack
/// verification failure. The borrowed input pack remains unchanged on failure.
pub fn build_st_cloud_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    station_json: &str,
    gyms_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let (town, raw) = crate::st_cloud_design::parse(town_json, station_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnStCloud"
            && town.native_constant == "MN_ST_CLOUD"
            && town.compiled_tileset == "johto_modern"
            && town.dimensions_metatiles == [24, 18]
            && layout.id == town.id
            && layout.compiled_tileset == town.compiled_tileset
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && town.doors.len() == 4
            && town.interiors.len() == 4
            && !base.data.maps.contains_key(&town.id),
        "St Cloud identity, complete inventory or duplicate installation"
    );
    let mut pack = base.clone();
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    let room = town
        .interiors
        .iter()
        .find(|r| r.id == "MnStCloudGym")
        .context("canonical gym reference")?;
    let reference = raw["interiors"]
        .as_array()
        .context("room sources")?
        .iter()
        .find(|r| r["id"] == room.id)
        .context("gym source")?;
    let gym = canonical_city_gym::bind(
        &pack,
        room,
        &reference["source_reference"],
        gyms_json,
        Binding {
            slot: 5,
            town: "MnStCloud",
            constant: "MN_ST_CLOUD",
            return_warp: 6,
        },
    )?;
    let clerks = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .filter(|a| a.service.as_deref() == Some("pokemart"))
        .collect::<Vec<_>>();
    ensure!(
        clerks.len() == 1 && !pack.data.marts.0.contains_key("MART_MN_ST_CLOUD"),
        "one unoccupied St Cloud mart"
    );
    pack.data
        .marts
        .0
        .insert("MART_MN_ST_CLOUD".into(), clerks[0].stock.clone());
    let mut city = authoring::blank(&pack, "GoldenrodCity", &town.id, &town.native_constant)?;
    city.attributes.width = 24;
    city.attributes.height = 18;
    city.attributes.border_block = 5;
    city.attributes.tileset_name = town.compiled_tileset.clone();
    city.blocks = layout.compile()?;
    ordinary(&pack, &mut city, &town.residents, &town.signs)?;
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("paired public room")?;
        ensure!(
            usize::from(door.warp_index) == city.events.warps.len() + 1
                && door.return_warp_index == Some(door.warp_index + 1)
                && door.target_warp == i16::try_from(room.passive_arrival.index)?
                && door.arrival_tile == room.passive_arrival.tile,
            "St Cloud public door/passive warp pair differs"
        );
        authoring::warp(
            &mut city,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            door.target_warp,
        );
        authoring::warp(
            &mut city,
            (door.return_tile[0], door.return_tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("return")?)?,
        );
    }
    let mut modules = vec![(city, [24, 32])];
    for room in &town.interiors {
        let mut module = if room.id == "MnStCloudGym" {
            gym.clone()
        } else {
            crate::authored_town_rooms::build(
                &pack,
                &town,
                room,
                "MN_ST_CLOUD_POKECENTER_1F",
                "MART_MN_ST_CLOUD",
            )?
        };
        crate::authored_town_scripts::finish(&pack, &mut module, &town)?;
        modules.push((module, room.passive_arrival.tile));
    }
    for (map, _) in &mut modules {
        crate::authored_town_scripts::finish(&pack, map, &town)?;
    }
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        &town,
        &modules,
        &BTreeSet::from([[24, 0], [25, 0], [24, 35], [25, 35]]),
    )?;
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_ST_CLOUD",
            group_id: 77,
            landmark: "LANDMARK_MN_ST_CLOUD",
            fallback_landmark: Some("LANDMARK_MN_RIVER_BLUFFS"),
            constant: "MN_ST_CLOUD",
            name: "ST CLOUD",
            position: [88, 88],
            manifest: "minnesota-st-cloud-city-v1",
            town: "MnStCloud",
            clinic: "MnStCloudPokecenter1F",
            gym: "MnStCloudGym",
        },
    )?;
    crate::st_cloud_connections::install(&mut pack)?;
    pack.report
        .manifests
        .push("minnesota-st-cloud-city-v1".into());
    pack.report.maps = pack.data.maps.len();
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

fn ordinary(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    actors: &[crate::authored_town_design::Actor],
    signs: &[crate::authored_route_design::Sign],
) -> Result<()> {
    crate::authored_town_scripts::ordinary(
        pack,
        map,
        actors,
        signs,
        "MN_ST_CLOUD_POKECENTER_1F",
        "MART_MN_ST_CLOUD",
    )
}
