//! St Paul's native city doors and preserved canonical fourth-gym pad network.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::authored_route_layout::Layout;
use crate::authored_town_publication::{self, CityIdentity};
use crate::canonical_city_gym::{self, Binding};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Publishes St Paul, its public rooms and the canonical fourth gym.
///
/// The native gym pad network, trainers, grille, badge and TM stay intact.
/// Only its first three warp destinations and landmark change. Ordinary care,
/// PC and shopping use production services. A reciprocal public R04 seam is
/// installed and checked; no badge or story progress is granted. Depot rooms
/// are accessible, but the courier quest and rail service remain separate work.
/// Existing HOME, recovery, unrelated maps, source art and audio are preserved.
///
/// # Errors
/// Returns an error for stale sources/gym content, occupied identities, unsafe
/// geometry, invalid native scripts/doors, incorrect edge projections or pack
/// verification failure. The borrowed input pack remains unchanged on failure.
pub fn build_st_paul_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    depot_json: &str,
    gyms_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let (town, raw) = crate::st_paul_design::parse(town_json, depot_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnStPaul"
            && town.native_constant == "MN_ST_PAUL"
            && town.compiled_tileset == "kanto"
            && town.dimensions_metatiles == [32, 24]
            && layout.id == town.id
            && layout.compiled_tileset == town.compiled_tileset
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && town.doors.len() == 5
            && town.interiors.len() == 5
            && !base.data.maps.contains_key(&town.id),
        "St Paul identity, complete inventory or duplicate installation"
    );
    let mut pack = base.clone();
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    let room = town
        .interiors
        .iter()
        .find(|r| r.id == "MnStPaulGym")
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
            slot: 4,
            town: "MnStPaul",
            constant: "MN_ST_PAUL",
            return_warp: 10,
        },
    )?;
    let clerks = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .filter(|a| a.service.as_deref() == Some("pokemart"))
        .collect::<Vec<_>>();
    ensure!(
        clerks.len() == 1 && !pack.data.marts.0.contains_key("MART_MN_ST_PAUL"),
        "one unoccupied St Paul mart"
    );
    pack.data
        .marts
        .0
        .insert("MART_MN_ST_PAUL".into(), clerks[0].stock.clone());
    let mut city = authoring::blank(&pack, "SaffronCity", &town.id, &town.native_constant)?;
    city.attributes.width = 32;
    city.attributes.height = 24;
    city.attributes.border_block = 0x0f;
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
            "St Paul public door/passive warp pair differs"
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
    let mut modules = vec![(city, [6, 24])];
    for room in &town.interiors {
        let mut module = if room.id == "MnStPaulGym" {
            gym.clone()
        } else {
            crate::authored_town_rooms::build(
                &pack,
                &town,
                room,
                "MN_ST_PAUL_POKECENTER_1F",
                "MART_MN_ST_PAUL",
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
        &BTreeSet::from([[0, 24], [0, 25]]),
    )?;
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_ST_PAUL",
            group_id: 76,
            landmark: "LANDMARK_MN_ST_PAUL",
            fallback_landmark: Some("LANDMARK_MN_TWIN_CITIES"),
            constant: "MN_ST_PAUL",
            name: "ST PAUL",
            position: [108, 104],
            manifest: "minnesota-st-paul-city-v1",
            town: "MnStPaul",
            clinic: "MnStPaulPokecenter1F",
            gym: "MnStPaulGym",
        },
    )?;
    crate::st_paul_connections::install(&mut pack)?;
    pack.report
        .manifests
        .push("minnesota-st-paul-city-v1".into());
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
        "MN_ST_PAUL_POKECENTER_1F",
        "MART_MN_ST_PAUL",
    )
}
