//! Moorhead's native levee neighborhood, public clinic and prairie supplies.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_geometry::{kind, reachable};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{moorhead_connections, moorhead_story};

const MANIFEST: &str = "minnesota-moorhead-v1";
const CLINIC: &str = "MnMoorheadPokecenter1F";
const CLINIC_CONSTANT: &str = "MN_MOORHEAD_POKECENTER_1F";
const MART: &str = "MART_MN_MOORHEAD";

/// Publishes Moorhead and its two reserved non-gym interiors over a verified pack.
///
/// Installs native free care, PC, remembered blackout recovery, priced shopping
/// and the levee notebook's once-only FULL HEAL. The marker must actually be read;
/// a full Pack retains the gift for retry. A continuous native bank and fence
/// protect the riverside walk. Both R16 walking lanes connect in both directions;
/// all old route blocks, residents, trainers, habitat and scripts remain exact.
/// Original maps, HOME, recovery records, art, items and audio are preserved.
///
/// # Errors
/// Returns an error for malformed or incomplete source, occupied identities,
/// unsupported art, stale R16 connections, unsafe rooms/counters/doors/levee,
/// weakened story requirements or failed native script/pack verification.
pub fn build_moorhead_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let raw: Value = serde_json::from_str(town_json).context("Moorhead dossier")?;
    let town: Town = serde_json::from_value(raw.clone()).context("native Moorhead source")?;
    let layout: Layout = serde_json::from_str(layout_json).context("Moorhead layout")?;
    ensure!(
        town.id == "MnMoorhead"
            && town.native_constant == "MN_MOORHEAD"
            && town.compiled_tileset == "johto_modern"
            && town.dimensions_metatiles == [16, 12]
            && layout.id == town.id
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset
            && layout.schema_version == 2
            && town.doors.len() == 2
            && town.interiors.len() == 2
            && town
                .interiors
                .iter()
                .map(|r| r.id.as_str())
                .eq([CLINIC, "MnMoorheadMart"])
            && town.ports.len() == 1
            && town.ports[0].route == "MN_R16"
            && town.ports[0].side == "E"
            && town.ports[0].gate_metatile == [14, 6]
            && town.ports[0].landing_metatile == [12, 6]
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "complete uninstalled Moorhead and both reserved rooms required"
    );
    moorhead_story::validate(&raw["quest_contract"])?;
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        layout_json,
        "johto_modern",
        "johto_modern",
        &town.id,
    )?;
    for special in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(special)?;
    }
    ensure!(
        town.interiors[0].pc.is_some()
            && town.interiors[0].dimensions_metatiles == [5, 4]
            && town.interiors[0].passive_arrival.tile == [3, 6]
            && town.interiors[0]
                .residents
                .iter()
                .filter(|a| a.service.as_deref() == Some("heal_party"))
                .count()
                == 1,
        "Moorhead clinic needs native care, PC and compact passive recovery"
    );
    crate::authored_city_expansion_events::flags(
        &mut pack,
        &[moorhead_story::READ.into(), moorhead_story::GIVEN.into()],
    )?;
    let clerks = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .filter(|a| a.service.as_deref() == Some("pokemart"))
        .collect::<Vec<_>>();
    ensure!(
        clerks.len() == 1
            && clerks[0].stock.iter().map(String::as_str).eq([
                "GREAT_BALL",
                "SUPER_POTION",
                "ANTIDOTE",
                "ICE_HEAL",
                "ESCAPE_ROPE",
                "REPEL",
            ])
            && clerks[0]
                .stock
                .iter()
                .all(|id| pack.data.items.contains_key(id)),
        "one native stocked Moorhead shop required"
    );
    ensure!(
        pack.data
            .marts
            .0
            .insert(MART.into(), clerks[0].stock.clone())
            .is_none(),
        "occupied Moorhead mart"
    );
    let mut exterior = authoring::blank(&pack, "AzaleaTown", &town.id, &town.native_constant)?;
    exterior.attributes.width = 16;
    exterior.attributes.height = 12;
    exterior.attributes.border_block = 5;
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    crate::regional_town_fishing::freshwater(&pack, &mut exterior)?;
    exterior.blocks = layout.compile()?;
    crate::authored_town_scripts::ordinary(
        &pack,
        &mut exterior,
        &town.residents,
        &town.signs,
        CLINIC_CONSTANT,
        MART,
    )?;
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("paired Moorhead room")?;
        ensure!(
            usize::from(door.warp_index) == exterior.events.warps.len() + 1
                && door.return_warp_index == Some(door.warp_index + 1)
                && door.target_warp == 3
                && door.arrival_tile == room.passive_arrival.tile,
            "Moorhead active/passive facade binding differs"
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
            i16::try_from(door.return_warp_index.context("Moorhead return")?)?,
        );
    }
    moorhead_story::marker(&mut exterior)?;
    crate::authored_town_scripts::finish(&pack, &mut exterior, &town)?;
    let mut modules = vec![(exterior, [30, 12])];
    for room in &town.interiors {
        let mut map = crate::authored_town_rooms::build(&pack, &town, room, CLINIC_CONSTANT, MART)?;
        if room.id == CLINIC {
            moorhead_story::volunteer(&mut map, &raw["quest_contract"])?;
            crate::authored_town_scripts::finish(&pack, &mut map, &town)?;
        }
        for actor in &room.residents {
            if let (Some(counter), Some(approach)) = (actor.counter_tile, actor.approach) {
                ensure!(
                    (0..2).all(|i| u32::from(actor.tile[i]) + u32::from(approach[i])
                        == 2 * u32::from(counter[i]))
                        && !map.events.bg_events.iter().any(|e| [e.x, e.y] == counter),
                    "Moorhead counter must align with its real actor and remain unshadowed"
                );
            }
        }
        modules.push((map, room.passive_arrival.tile));
    }
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        &town,
        &modules,
        &BTreeSet::from([[31, 12], [31, 13]]),
    )?;
    let exterior = &modules[0].0;
    for y in 0..24 {
        ensure!(
            kind(&pack, exterior, [9, y])? == "BUOY",
            "Moorhead needs the continuous native stone riverbank"
        );
    }
    for y in 2..22 {
        ensure!(
            kind(&pack, exterior, [10, y])? == "WALL" && kind(&pack, exterior, [11, y])? == "FLOOR",
            "Moorhead needs an unbroken levee rail and passing path"
        );
    }
    for p in reachable(&pack, exterior, [30, 12], true)? {
        ensure!(
            kind(&pack, exterior, p)? != "WATER",
            "Moorhead levee must not expose the unbound river to Surf"
        );
    }
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_MOORHEAD",
            group_id: 87,
            landmark: "LANDMARK_MN_MOORHEAD",
            fallback_landmark: Some("LANDMARK_MN_RED_RIVER_PRAIRIE"),
            constant: "MN_MOORHEAD",
            name: "MOORHEAD",
            position: [24, 84],
            manifest: MANIFEST,
            town: "MnMoorhead",
            clinic: CLINIC,
            gym: "",
        },
    )?;
    moorhead_connections::install(&mut pack)?;
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
