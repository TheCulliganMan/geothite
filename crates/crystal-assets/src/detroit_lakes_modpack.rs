//! Detroit Lakes' authored public rooms, lake frontage and observation reward.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_geometry::{kind, reachable};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, detroit_lakes_connections, detroit_lakes_quest};

const MANIFEST: &str = "minnesota-detroit-lakes-v1";
const CLINIC: &str = "MnDetroitLakesPokecenter1F";
const MART: &str = "MART_MN_DETROIT_LAKES";

/// Publishes Detroit Lakes and all five of its non-gym room reservations.
///
/// Installs native free care/PC/recovery, priced shopping and the retryable SQ01
/// observation hand-in. The original wooded approach, trainer activities and
/// route programs remain intact; reciprocal roads join the town to R15 and that
/// approach. R19 retains its existing Lake Itasca connection. The public lake has a real launch and closed Surf edges.
/// The station's public hall and residents are installed; five-stop Glacier Line
/// transport remains separate unfinished work. No train or chapter flags are
/// granted. All original audio, HOME and existing recovery records are retained.
///
/// # Errors
/// Returns an error for malformed/incomplete source, conflicting map/art/group
/// identities, stale road preimages, unsafe rooms/counters/doors, altered SQ01
/// requirements, or failed native script or pack verification. Inputs are borrowed.
pub fn build_detroit_lakes_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    station_json: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let (town, raw) = parse(town_json, station_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnDetroitLakes"
            && town.native_constant == "MN_DETROIT_LAKES"
            && town.compiled_tileset == "mn_detroit_lakes"
            && town.dimensions_metatiles == [24, 18]
            && layout.id == town.id
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset
            && layout.schema_version == 2
            && town.doors.len() == 5
            && town.interiors.len() == 5
            && !base.report.manifests.iter().any(|id| id == MANIFEST)
            && !base.data.maps.contains_key(&town.id),
        "complete uninstalled Detroit Lakes source required"
    );
    let expected = [
        CLINIC,
        "MnDetroitLakesMart",
        "MnDetroitLakesStation",
        "MnDetroitLakesCanoeCabin",
        "MnDetroitLakesRangerCabin",
    ];
    ensure!(
        town.interiors.iter().map(|r| r.id.as_str()).eq(expected),
        "all five reserved Detroit rooms required in source order"
    );
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        layout_json,
        &town.compiled_tileset,
        "johto",
        &town.id,
    )?;
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    for flag in [
        "ENGINE_MINERALBADGE",
        "EVENT_MN_LAKE_OBSERVATION_01",
        "EVENT_MN_LAKE_OBSERVATION_02",
        "EVENT_MN_LAKE_OBSERVATION_03",
    ] {
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(flag),
            "original observation prerequisite missing: {flag}"
        );
    }
    crate::authored_city_expansion_events::flags(
        &mut pack,
        &[detroit_lakes_quest::COMPLETED.into()],
    )?;
    let clerks = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .filter(|a| a.service.as_deref() == Some("pokemart"))
        .collect::<Vec<_>>();
    ensure!(
        clerks.len() == 1
            && !clerks[0].stock.is_empty()
            && clerks[0]
                .stock
                .iter()
                .all(|item| pack.data.items.contains_key(item)),
        "one native stocked Detroit clerk required"
    );
    ensure!(
        pack.data
            .marts
            .0
            .insert(MART.into(), clerks[0].stock.clone())
            .is_none(),
        "occupied Detroit mart"
    );
    let mut exterior = authoring::blank(&pack, "GoldenrodCity", &town.id, &town.native_constant)?;
    exterior.attributes.width = 24;
    exterior.attributes.height = 18;
    exterior.attributes.border_block = 5;
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    crate::regional_town_fishing::freshwater(&pack, &mut exterior)?;
    exterior.blocks = layout.compile()?;
    crate::authored_town_scripts::ordinary(
        &pack,
        &mut exterior,
        &town.residents,
        &town.signs,
        "MN_DETROIT_LAKES_POKECENTER_1F",
        MART,
    )?;
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("paired Detroit room")?;
        ensure!(
            usize::from(door.warp_index) == exterior.events.warps.len() + 1
                && door.return_warp_index == Some(door.warp_index + 1)
                && door.target_warp == 3
                && door.arrival_tile == room.passive_arrival.tile,
            "Detroit active/passive door pairing differs"
        );
        authoring::warp(
            &mut exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            3,
        );
        authoring::warp(
            &mut exterior,
            (door.return_tile[0], door.return_tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("return")?)?,
        );
    }
    crate::authored_town_scripts::finish(&pack, &mut exterior, &town)?;
    let mut modules = vec![(exterior, [6, 18])];
    for room in &town.interiors {
        let mut map = crate::authored_town_rooms::build(
            &pack,
            &town,
            room,
            "MN_DETROIT_LAKES_POKECENTER_1F",
            MART,
        )?;
        if room.id == "MnDetroitLakesRangerCabin" {
            detroit_lakes_quest::install(&mut map, &raw["quest_contract"])?;
            crate::authored_town_scripts::finish(&pack, &mut map, &town)?;
        }
        for actor in &room.residents {
            if let (Some(counter), Some(approach)) = (actor.counter_tile, actor.approach) {
                ensure!(
                    actor.tile[0] + approach[0] == 2 * counter[0]
                        && actor.tile[1] + approach[1] == 2 * counter[1]
                        && !map.events.bg_events.iter().any(|e| [e.x, e.y] == counter),
                    "native counter must be aligned and free of shadowing furniture"
                );
            }
        }
        modules.push((map, room.passive_arrival.tile));
    }
    let edges = BTreeSet::from([[0, 18], [0, 19], [47, 18], [47, 19]]);
    crate::authored_town_geometry::validate_with_edges(&pack, &town, &modules, &edges)?;
    let station = &modules[3].0;
    let public = reachable(&pack, station, [11, 17], false)?;
    ensure!(
        !public.contains(&[6, 5])
            && !public.contains(&[11, 5])
            && kind(&pack, station, [6, 5])? == "DOOR"
            && kind(&pack, station, [11, 5])? == "DOOR",
        "station conductor must seal both train doors"
    );
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_DETROIT_LAKES",
            group_id: 86,
            landmark: "LANDMARK_MN_DETROIT_LAKES",
            fallback_landmark: Some("LANDMARK_MN_CENTRAL_LAKES"),
            constant: "MN_DETROIT_LAKES",
            name: "DETROIT LAKES",
            position: [48, 88],
            manifest: MANIFEST,
            town: "MnDetroitLakes",
            clinic: CLINIC,
            gym: "",
        },
    )?;
    detroit_lakes_connections::install(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::validate_compact_superior_geography(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn parse(town: &str, station: &str) -> Result<(Town, Value)> {
    let mut raw: Value = serde_json::from_str(town).context("Detroit town dossier")?;
    let mut station: Value = serde_json::from_str(station).context("Detroit station dossier")?;
    ensure!(
        station["id"] == "MnDetroitLakesStation"
            && station["dimensions_metatiles"] == json!([12, 10])
            && station["passive_arrival"]["tile"] == json!([11, 17]),
        "exact public Detroit station required"
    );
    station["template"] = json!("GoldenrodMagnetTrainStation");
    station["border_block"] = json!(15);
    for room in raw["interiors"].as_array_mut().context("Detroit rooms")? {
        if room["id"] == "MnDetroitLakesStation" {
            *room = station.clone();
        }
    }
    let rooms = raw["interiors"]
        .as_array()
        .context("Detroit room bindings")?
        .clone();
    for door in raw["doors"].as_array_mut().context("Detroit facades")? {
        let room = rooms
            .iter()
            .find(|r| r["id"] == door["target"])
            .context("Detroit facade destination")?;
        door["arrival_tile"] = room["passive_arrival"]["tile"].clone();
    }
    Ok((
        serde_json::from_value(raw.clone()).context("native Detroit contract")?,
        raw,
    ))
}
