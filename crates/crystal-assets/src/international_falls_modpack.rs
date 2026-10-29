//! International Falls' winter services and guarded SQ03 ranger circuit.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_geometry::{kind, reachable};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{international_falls_connections, international_falls_story};

const MANIFEST: &str = "minnesota-international_falls-v1";
const CLINIC: &str = "MnInternationalFallsPokecenter1F";
const CLINIC_CONSTANT: &str = "MN_INTERNATIONAL_FALLS_POKECENTER_1F";
const MART: &str = "MART_MN_INTERNATIONAL_FALLS";

/// Publishes International Falls, its two services and the native SQ03 ranger desk.
///
/// Adds free care/PC/recovery, priced winter supplies, a remembered check-in and
/// one retryable PP Up after a genuine eastbound Voyageurs arrival at basecamp.
/// The actual Rainy River outlet requires all eight badges, report and signal;
/// inbound travelers retain access to public services. Existing R20, ice puzzles,
/// audio, source art, gyms and HOME are preserved.
///
/// # Errors
/// Returns an error for incomplete source, occupied identities, stale northern
/// endpoints, unsafe native geometry, weakened quest guards or pack verification.
pub fn build_international_falls_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let raw: Value = serde_json::from_str(town_json).context("InternationalFalls dossier")?;
    let town: Town =
        serde_json::from_value(raw.clone()).context("native InternationalFalls source")?;
    let layout: Layout = serde_json::from_str(layout_json).context("InternationalFalls layout")?;
    ensure!(
        town.id == "MnInternationalFalls"
            && town.native_constant == "MN_INTERNATIONAL_FALLS"
            && town.compiled_tileset == "mn_international_falls"
            && town.dimensions_metatiles == [16, 14]
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
                .eq([CLINIC, "MnInternationalFallsMart"])
            && town.ports.len() == 1
            && town.ports[0].route == "MN_R17"
            && town.ports[0].side == "S"
            && town.ports[0].gate_metatile == [10, 12]
            && town.ports[0].landing_metatile == [10, 10]
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "complete uninstalled InternationalFalls and both reserved rooms required"
    );
    international_falls_story::validate(&raw["quest_contract"])?;
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        layout_json,
        "mn_international_falls",
        "johto_modern",
        &town.id,
    )?;
    for special in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(special)?;
    }
    ensure!(
        town.interiors[0].pc.is_some()
            && town.interiors[0].dimensions_metatiles == [5, 5]
            && town.interiors[0].passive_arrival.tile == [3, 8]
            && town.interiors[0]
                .residents
                .iter()
                .filter(|a| a.service.as_deref() == Some("heal_party"))
                .count()
                == 1,
        "InternationalFalls clinic needs native care, PC and compact passive recovery"
    );
    crate::authored_city_expansion_events::flags(
        &mut pack,
        &[
            international_falls_story::REPORT.into(),
            international_falls_story::ARRIVED.into(),
            international_falls_story::GIVEN.into(),
        ],
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
                "ULTRA_BALL",
                "HYPER_POTION",
                "FULL_HEAL",
                "ICE_HEAL",
                "ESCAPE_ROPE",
                "MAX_REPEL",
            ])
            && clerks[0]
                .stock
                .iter()
                .all(|id| pack.data.items.contains_key(id)),
        "one native stocked InternationalFalls shop required"
    );
    ensure!(
        pack.data
            .marts
            .0
            .insert(MART.into(), clerks[0].stock.clone())
            .is_none(),
        "occupied InternationalFalls mart"
    );
    let mut exterior = authoring::blank(&pack, "AzaleaTown", &town.id, &town.native_constant)?;
    exterior.attributes.width = 16;
    exterior.attributes.height = 14;
    exterior.attributes.border_block = 0x81;
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
            .context("paired InternationalFalls room")?;
        ensure!(
            usize::from(door.warp_index) == exterior.events.warps.len() + 1
                && door.return_warp_index == Some(door.warp_index + 1)
                && door.target_warp == 3
                && door.arrival_tile == room.passive_arrival.tile,
            "InternationalFalls active/passive facade binding differs"
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
            i16::try_from(
                door.return_warp_index
                    .context("InternationalFalls return")?,
            )?,
        );
    }
    crate::authored_town_scripts::finish(&pack, &mut exterior, &town)?;
    let mut modules = vec![(exterior, [20, 25])];
    for room in &town.interiors {
        let mut map = crate::authored_town_rooms::build(&pack, &town, room, CLINIC_CONSTANT, MART)?;
        if room.id == CLINIC {
            international_falls_story::ranger(&mut map, &raw["quest_contract"])?;
            crate::authored_town_scripts::finish(&pack, &mut map, &town)?;
        }
        for actor in &room.residents {
            if let (Some(counter), Some(approach)) = (actor.counter_tile, actor.approach) {
                ensure!(
                    (0..2).all(|i| u32::from(actor.tile[i]) + u32::from(approach[i])
                        == 2 * u32::from(counter[i]))
                        && !map.events.bg_events.iter().any(|e| [e.x, e.y] == counter),
                    "InternationalFalls counter must align with its real actor and remain unshadowed"
                );
            }
        }
        modules.push((map, room.passive_arrival.tile));
    }
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        &town,
        &modules,
        &BTreeSet::from([[20, 27], [21, 27]]),
    )?;
    for p in reachable(&pack, &modules[0].0, [20, 25], true)? {
        ensure!(
            kind(&pack, &modules[0].0, p)? != "WATER",
            "Falls has no unbound Surf exit"
        );
    }
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_INTERNATIONAL_FALLS",
            group_id: 88,
            landmark: "LANDMARK_MN_INTERNATIONAL_FALLS",
            fallback_landmark: Some("LANDMARK_MN_BORDER_PINES"),
            constant: "MN_INTERNATIONAL_FALLS",
            name: "INTERNATIONAL_FALLS",
            position: [80, 8],
            manifest: MANIFEST,
            town: "MnInternationalFalls",
            clinic: CLINIC,
            gym: "",
        },
    )?;
    international_falls_connections::install(&mut pack)?;
    international_falls_story::checkpoint(&mut pack, &town, &raw["quest_contract"])?;
    international_falls_story::arrival(&mut pack, &town)?;
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
