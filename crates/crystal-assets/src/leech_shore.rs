//! Native shore settlement and main land journey from Brainerd to Bemidji.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_publication::{CityIdentity, publish};
use crate::grand_marais_modpack::authoring;

const BASE: &str = "d7ad6d24e6716c7d594044c056046f08cbb57ca0943ea6443e0e422a9b00baae";
const MANIFEST: &str = "minnesota-leech-shore-v1";
const CLINIC: &str = "MN_LEECH_LAKE_CARE_CABIN";

/// Publishes Leech Lake's cabins, witnesses, care and both connected land roads.
///
/// Lake alone permits departure from Brainerd and Leech Lake. The optional
/// ranger report/Surf gift is never required. Inbound travel remains free.
/// Existing route encounters, gyms, story, art, audio and recovery stay exact;
/// this adds only the shore settlement's own native recovery records.
///
/// # Errors
/// Returns an error for stale/duplicate input, occupied native identities,
/// changed dimensions/prerequisites, unsafe doors or walking/Surf boundaries,
/// invalid script lowering, incorrect connections or failed pack verification.
/// The borrowed input remains unchanged.
pub fn build_leech_shore_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    r07_json: &str,
    r08_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == BASE && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact lake-country input required; duplicate/stale shore publication"
    );
    let town: Town = serde_json::from_str(town_json)?;
    let raw: Value = serde_json::from_str(town_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnLeechLake"
            && town.native_constant == "MN_LEECH_LAKE_SHORE"
            && town.dimensions_metatiles == [32, 24]
            && layout.dimensions_metatiles == [32, 24]
            && layout.id == town.id
            && layout.schema_version == 2
            && town.compiled_tileset == "mn_headwaters"
            && layout.compiled_tileset == town.compiled_tileset
            && town.interiors.len() == 2
            && town.doors.len() == 2
            && !base.data.maps.contains_key(&town.id),
        "complete native Leech Lake contract required"
    );
    for (source, name) in [(r07_json, "MN_R07"), (r08_json, "MN_R08")] {
        let route: Value = serde_json::from_str(source)?;
        ensure!(
            route["id"] == name && route["required_flags"] == serde_json::json!(["BADGE_6"]),
            "Lake-only land roads required; no Surf/report gate"
        );
    }
    let mut pack = base.clone();
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    let mut map = authoring::blank(&pack, "CherrygroveCity", &town.id, &town.native_constant)?;
    map.attributes.width = 32;
    map.attributes.height = 24;
    map.attributes.border_block = 5;
    map.attributes.tileset_name = town.compiled_tileset.clone();
    map.blocks = layout.compile()?;
    crate::leech_shore_geometry::validate(&pack, &town, &map, &raw)?;
    crate::authored_town_scripts::ordinary(
        &pack,
        &mut map,
        &town.residents,
        &town.signs,
        CLINIC,
        "",
    )?;
    for (index, door) in town.doors.iter().enumerate() {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("real shore room")?;
        ensure!(
            usize::from(door.warp_index) == index * 2 + 1
                && door.return_warp_index == Some(door.warp_index + 1)
                && door.target_warp == 3
                && room.passive_arrival.index == 3
                && door.arrival_tile == room.passive_arrival.tile,
            "native shore door/passive pairing"
        );
        authoring::warp(
            &mut map,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            3,
        );
        authoring::warp(
            &mut map,
            (door.return_tile[0], door.return_tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("passive exterior return")?)?,
        );
    }
    let mut modules = vec![(map, [32, 44])];
    for room in &town.interiors {
        let map = crate::authored_town_rooms::build(&pack, &town, room, CLINIC, "")?;
        modules.push((map, room.passive_arrival.tile));
    }
    for (map, _) in &mut modules {
        crate::authored_town_scripts::finish(&pack, map, &town)?;
    }
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        &town,
        &modules,
        &BTreeSet::from([[32, 0], [33, 0], [32, 47], [33, 47]]),
    )?;
    publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_LEECH_SHORE",
            group_id: 78,
            landmark: "LANDMARK_MN_LEECH_SHORE",
            fallback_landmark: Some("LANDMARK_MN_LEECH_LAKE"),
            constant: "MN_LEECH_LAKE_SHORE",
            name: "LEECH LAKE",
            position: [84, 48],
            manifest: MANIFEST,
            town: "MnLeechLake",
            clinic: "MnLeechLakeCareCabin",
            gym: "",
        },
    )?;
    crate::leech_shore_connections::install(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
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
