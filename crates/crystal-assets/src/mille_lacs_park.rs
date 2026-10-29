//! A source-authored Mille Lacs park joined to the existing Shore Watch route.
#[path = "mille_lacs_park_geometry.rs"]
mod geometry;
#[path = "mille_lacs_park_preservation.rs"]
mod preservation;
#[cfg(test)]
#[path = "mille_lacs_park_tests.rs"]
mod tests;

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_route_design::Pickup;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;

const MAP: &str = "MnMilleLacs";
const ROUTE: &str = "MnR18C01R00";
const CONSTANT: &str = "MN_MILLE_LACS";
const MANIFEST: &str = "minnesota-mille-lacs-park-v1";
const PICKUP: &str = "EVENT_MN_MILLE_LACS_ROOST";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    id: String,
    parent_content_hash: String,
    layout_sha256: String,
    town_sha256: String,
    route_module_sha256: String,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Adds the hand-authored park and a reciprocal road to R18's eastern shore.
///
/// Preserves the existing Shore Watch, badge gate and Good Rod authority. The
/// park has ordinary residents, signs, a grass-gated lookout with a short
/// downward return, a railed dock and an independent Surf-only PP Up. All old
/// maps remain exact except R18's declared connection and its attribute mirror.
/// No art, audio, trainer party, species, original flag or recovery is replaced.
/// The original HOME remains the New Game destination.
///
/// # Errors
/// Returns an error for an unreviewed parent/source hash, duplicate identity,
/// stale R18 preimage, unsafe terrain/ledge/dock/Surf boundary, invalid native
/// scripts, unavailable map identifiers or failed strict pack validation.
pub fn build_mille_lacs_park(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let contract: Contract = serde_json::from_str(contract_json).context("park contract")?;
    ensure!(
        contract.schema_version == 1
            && contract.id == "CURRENT_MILLE_LACS_PARK_V01"
            && contract.parent_content_hash == base.identity.content_hash
            && contract.town_sha256 == hash(town_json.as_bytes())
            && contract.layout_sha256 == hash(layout_json.as_bytes()),
        "unreviewed Mille Lacs parent or authored sources"
    );
    let route = base.data.maps.get(ROUTE).context("original R18 terminus")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(route)?)?) == contract.route_module_sha256,
        "R18 native preimage differs"
    );
    ensure!(
        !base.data.maps.contains_key(MAP) && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Mille Lacs park is already installed"
    );
    let raw: Value = serde_json::from_str(town_json)?;
    let town: Town = serde_json::from_value(raw.clone())?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    let items: Vec<Pickup> = serde_json::from_value(raw["items"].clone())?;
    ensure!(
        town.id == MAP
            && town.native_constant == CONSTANT
            && town.compiled_tileset == "mn_headwaters"
            && town.dimensions_metatiles == [24, 20]
            && town.interiors.is_empty()
            && town.doors.is_empty()
            && town.residents.len() == 5
            && town.residents.iter().all(|a| a.service.is_none())
            && layout.id == MAP
            && layout.schema_version == 2
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset
            && items.len() == 1
            && items[0].id == "MN_ML_PARK_ROOST"
            && items[0].flag == PICKUP
            && items[0].item == "PP_UP"
            && items[0].quantity == 1
            && items[0].tile == [36, 29],
        "complete reviewed park, ordinary residents and independent roost reward required"
    );
    let mut pack = base.clone();
    crate::authored_city_expansion_events::flags(&mut pack, &[PICKUP.into()])?;
    let mut map = authoring::blank(&pack, ROUTE, MAP, CONSTANT)?;
    map.attributes.width = 24;
    map.attributes.height = 20;
    map.attributes.border_block = 5;
    map.blocks = layout.compile()?;
    crate::authored_town_scripts::ordinary(&pack, &mut map, &town.residents, &town.signs, "", "")?;
    crate::authored_city_expansion_events::activities(&mut pack, &mut map, &[], &items)?;
    crate::authored_town_scripts::finish(&pack, &mut map, &town)?;
    map.script_field_pickups = crate::parse_script_field_pickups(MAP, &map.scripts, &map.objects)?;
    ensure!(
        map.script_field_pickups.len() == 1,
        "roost needs its native item-ball handler"
    );
    let modules = vec![(map, [2, 12])];
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        &town,
        &modules,
        &BTreeSet::from([[0, 12], [0, 13]]),
    )?;
    geometry::validate(&pack, &modules[0].0)?;
    let group_id = (100..=255)
        .find(|id| {
            !pack
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == *id)
        })
        .context("free park map group")?;
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_MILLE_LACS_PARK",
            group_id: i16::try_from(group_id)?,
            landmark: "LANDMARK_MN_MILLE_LACS_PARK",
            fallback_landmark: Some("LANDMARK_MN_MILLE_LACS_SHORE"),
            constant: CONSTANT,
            name: "MILLE LACS",
            position: [104, 84],
            manifest: MANIFEST,
            town: MAP,
            clinic: "",
            gym: "",
        },
    )?;
    geometry::connect(&mut pack)?;
    let mut encounters = base
        .data
        .wild_encounters
        .get(ROUTE)
        .context("native R18 habitat")?
        .clone();
    ensure!(
        encounters.zones.is_empty() && encounters.swarm_overrides.is_empty(),
        "review park-local zones/swarms before inheriting R18 habitat"
    );
    encounters.map_name = MAP.into();
    ensure!(
        pack.data
            .wild_encounters
            .insert(MAP.into(), encounters)
            .is_none(),
        "occupied park habitat"
    );
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    preservation::validate(base, &pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

/// Rechecks the published park contracts for a later terrain-only descendant.
pub(crate) fn validate_current_terrain(
    pack: &CompiledGamePack,
    map: &crate::MapModule,
) -> Result<()> {
    geometry::validate(pack, map)
}
