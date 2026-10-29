//! A native public Winona depot with a real town door and remembered residents.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use serde::Deserialize;
use serde_json::Value;

use crate::authored_route_design::{Door, Sign};
use crate::authored_town_design::{Room, Town};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, MapScriptSectionCommand, authored_town_geometry, winona_station_story,
};

const MANIFEST: &str = "minnesota-winona-station-v1";

#[derive(Deserialize)]
struct Source {
    #[serde(flatten)]
    room: Room,
    exterior_binding: Binding,
    story_contract: Value,
}

#[derive(Deserialize)]
struct Binding {
    hub: String,
    expected_dimensions_metatiles: [u16; 2],
    door: Door,
    patches: Vec<Patch>,
    sign: Sign,
    retained_connections: Vec<MapConnection>,
    retained_edge_lanes: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
struct Patch {
    position: [u16; 2],
    before: String,
    after: String,
}

/// Adds Winona's public depot over the preserved Minnesota regional pack.
///
/// The existing town changes only at five explicitly declared native blocks,
/// two new warp records and one placard. The native first-entry callback records
/// this station alone. Reading its winter log and hearing its mason unlock one
/// retryable ICE HEAL from the porter. Passenger service remains closed pending
/// the separate five-stop Glacier Line binder. Existing story and audio survive.
///
/// # Errors
/// Returns an error for stale source/preimages, occupied map/flag identities,
/// malformed native facades/rooms, unsafe arrivals or unreachable encounters,
/// an unsealed boarding throat, weakened story contracts or invalid publication.
pub fn build_winona_station_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    station_json: &str,
) -> Result<CompiledGamePack> {
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Source = serde_json::from_str(station_json).context("Winona depot source")?;
    let mut town_value: Value = serde_json::from_str(town_json).context("Winona town source")?;
    let room_value: Value = serde_json::from_str(station_json)?;
    ensure!(
        source.room.id == "MnWinonaStation"
            && source.room.native_constant == "MN_WINONA_STATION"
            && source.room.tileset == "train_station"
            && source.room.template == "GoldenrodMagnetTrainStation"
            && source.room.dimensions_metatiles == [10, 9]
            && source.room.border_block == 15
            && source.room.passive_arrival.tile == [11, 15]
            && source.room.passive_arrival.index == 3
            && source.room.exits.len() == 2
            && source.room.residents.len() == 4
            && source.room.signs.len() == 1
            && source.exterior_binding.hub == "MnWinona"
            && source.exterior_binding.expected_dimensions_metatiles == [16, 14]
            && !base.data.maps.contains_key("MnWinonaStation")
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "one complete uninstalled native Winona depot required"
    );
    let binding = &source.exterior_binding;
    ensure!(
        binding.door.warp_index == 9
            && binding.door.return_warp_index == Some(10)
            && binding.door.tile == [19, 13]
            && binding.door.return_tile == [19, 14]
            && binding.door.arrival_tile == [11, 15]
            && binding.door.target == source.room.id
            && binding.door.target_warp == 3
            && binding.sign.tile == [22, 12]
            && binding.sign.id == "MN_WINONA_DEPOT_SIGN"
            && binding.patches.len() == 5,
        "Winona depot facade, passive return and sign anchors changed"
    );
    town_value["doors"]
        .as_array_mut()
        .context("town doors")?
        .push(room_value["exterior_binding"]["door"].clone());
    town_value["interiors"]
        .as_array_mut()
        .context("town rooms")?
        .push(room_value);
    let town: Town = serde_json::from_value(town_value).context("paired native town contract")?;
    ensure!(
        town.id == "MnWinona"
            && town.native_constant == "MN_WINONA"
            && town.compiled_tileset == "johto"
            && town.dimensions_metatiles == [16, 14],
        "original native Winona required"
    );
    let mut pack = base.clone();
    crate::authored_city_expansion_events::flags(
        &mut pack,
        &[
            winona_station_story::VISITED.into(),
            winona_station_story::READ.into(),
            winona_station_story::HEARD.into(),
            winona_station_story::GIVEN.into(),
        ],
    )?;
    let mut exterior = pack
        .data
        .maps
        .get("MnWinona")
        .context("installed Winona")?
        .clone();
    ensure!(
        exterior.attributes.width == 16
            && exterior.attributes.height == 14
            && exterior.attributes.tileset_name == "johto"
            && exterior.events.warps.len() == 8,
        "original Winona extent, art and eight native warps required"
    );
    let expected = [
        ([9, 5], 0x18),
        ([10, 5], 0x19),
        ([9, 6], 0x16),
        ([10, 6], 0x1e),
        ([11, 6], 0x45),
    ];
    for (patch, (position, block)) in binding.patches.iter().zip(expected) {
        let before = u16::from_str_radix(&patch.before, 16)?;
        let after = u16::from_str_radix(&patch.after, 16)?;
        ensure!(
            patch.position == position && before == 2 && after == block,
            "depot must keep the complete native house drawing and actual placard"
        );
        let index = usize::from(position[1]) * 16 + usize::from(position[0]);
        ensure!(
            exterior.blocks[index] == before,
            "changed depot clearing preimage"
        );
        exterior.blocks[index] = after;
    }
    authoring::warp(&mut exterior, (19, 13), "MN_WINONA_STATION", 3);
    authoring::warp(&mut exterior, (19, 14), "MN_WINONA", 10);
    let mut addition = authoring::blank(&pack, "NewBarkTown", "MnWinona", "MN_WINONA")?;
    crate::authored_town_scripts::ordinary(
        &pack,
        &mut addition,
        &[],
        std::slice::from_ref(&binding.sign),
        "MN_WINONA_POKECENTER_1F",
        "",
    )?;
    crate::authored_town_scripts::finish(&pack, &mut addition, &town)?;
    crate::authored_city_expansion::merge_new_content(&mut exterior, addition)?;
    let mut room = crate::authored_town_rooms::build(
        &pack,
        &town,
        &source.room,
        "MN_WINONA_POKECENTER_1F",
        "",
    )?;
    winona_station_story::install(&mut room, &source.story_contract)?;
    crate::authored_town_scripts::finish(&pack, &mut room, &town)?;
    room.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec![
                "MAPCALLBACK_NEWMAP".into(),
                "MnWinonaDepotFirstVisit".into(),
            ],
            command_index: room.map_script_section_commands.len(),
        });
    let retained_edges = crate::winona_station_edges::validate(&pack)?;
    ensure!(
        binding.retained_connections == exterior.attributes.connections
            && binding
                .retained_edge_lanes
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                == retained_edges
            && binding.retained_edge_lanes.len() == retained_edges.len(),
        "station source must retain the real road and all four river lanes"
    );
    // The shared geometry validator runs before connection installation. The
    // retained links have just passed real bidirectional native projections;
    // validate the unbound geometry view, then restore those exact links.
    let connections = std::mem::take(&mut exterior.attributes.connections);
    let modules = [(exterior, [19, 14]), (room, [11, 15])];
    authored_town_geometry::validate_with_edges(&pack, &town, &modules, &retained_edges)?;
    let [(mut exterior, _), (room, _)] = modules;
    exterior.attributes.connections = connections;
    let public = authored_town_geometry::reachable(&pack, &room, [11, 15], false)?;
    ensure!(
        [[6, 5], [11, 5], [11, 7]]
            .iter()
            .all(|p| !public.contains(p))
            && authored_town_geometry::kind(&pack, &room, [6, 5])? == "DOOR"
            && authored_town_geometry::kind(&pack, &room, [11, 5])? == "DOOR",
        "Winona conductor must seal the entire platform and both train doors"
    );
    let location = exterior
        .attributes
        .location
        .clone()
        .context("native Winona area")?;
    authored_town_publication::publish(
        &mut pack,
        vec![(room, [11, 15])],
        CityIdentity {
            group: "MINNESOTA_WINONA_STATION",
            group_id: 89,
            landmark: "LANDMARK_MN_WINONA_STATION",
            fallback_landmark: Some(&location),
            constant: "MN_WINONA_STATION",
            name: "WINONA DEPOT",
            position: [87, 87],
            manifest: MANIFEST,
            town: "MnWinona",
            clinic: "",
            gym: "",
        },
    )?;
    authoring::refresh_event_section(&mut exterior);
    authored_town_publication::replace(&mut pack, exterior)?;
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
