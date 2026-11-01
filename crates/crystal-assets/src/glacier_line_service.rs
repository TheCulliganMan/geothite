//! Guarded native passenger service in the five existing Minnesota station halls.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

pub(crate) const MANIFEST: &str = "minnesota-glacier-line-service-v1";
pub(crate) const SERVICE: &str = "EVENT_MN_COURIER_DEFEATED";
pub(crate) const PREFIX: &str = "MnGlacier";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ServiceSource {
    pub schema_version: u16,
    pub base_content_hash: String,
    pub service_event: String,
    pub stations: Vec<Station>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Station {
    pub id: String,
    pub map: String,
    pub constant: String,
    pub visited_flag: String,
    pub menu_label: String,
    pub conductor_script: String,
    pub conductor_core: [u16; 2],
    pub approach_core: [u16; 2],
    pub platform_core: [u16; 2],
    pub train_door_core: [u16; 2],
    pub other_train_door_core: [u16; 2],
    pub ride_arrival_core: [u16; 2],
    pub public_arrival_core: [u16; 2],
    pub public_arrival_warp: u16,
    pub parent_map: String,
    pub parent_entry_warp: u16,
    pub exit_warp_indices: Vec<u16>,
    pub blocks_sha256: String,
    pub conductor_open_steps: Vec<String>,
    pub player_board_steps: Vec<String>,
    pub conductor_restore_steps: Vec<String>,
}

impl Station {
    pub(crate) fn prefix(&self) -> String {
        format!("{PREFIX}{}", self.id)
    }
}

/// Opens free return travel after the existing Q03 courier victory.
///
/// Each destination must have been visited through its public entrance. The
/// native menu marks unknown stops, checks both guards again after selection,
/// restores the departure conductor, plays the supplied Magnet Train special,
/// and loads the destination's protected Floor. Existing halls, people, story
/// callbacks, ordinary dialogue, identities, catalogs, art and audio survive.
/// The dry halls use `FISHGROUP_NONE`, removing inherited unused fishing
/// references. Red Wing is not part of this five-stop service.
///
/// # Errors
/// Rejects stale or duplicate installation, different stops, unsafe native
/// anchors/movements, missing courier authority, occupied labels/events,
/// callback conflicts, incomplete typed tables or changed unrelated content.
pub fn build_glacier_line_service(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    let source: ServiceSource = serde_json::from_str(source_json)?;
    ensure!(
        source.schema_version == 1
            && source.base_content_hash == base.identity.content_hash
            && source.service_event == SERVICE,
        "exact current parent and Q03 courier authority required"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST)
            && !base.data.map_scripts.keys().any(|k| k.starts_with(PREFIX)),
        "Glacier Line is already installed or its namespace is occupied"
    );
    let ids = ["DETROIT_LAKES", "STAPLES", "ST_CLOUD", "ST_PAUL", "WINONA"];
    let maps = [
        "MnDetroitLakesStation",
        "MnStaplesStation",
        "MnStCloudStation",
        "MnStPaulUnionDepot",
        "MnWinonaStation",
    ];
    let constants = [
        "MN_DETROIT_LAKES_STATION",
        "MN_STAPLES_STATION",
        "MN_ST_CLOUD_STATION",
        "MN_ST_PAUL_UNION_DEPOT",
        "MN_WINONA_STATION",
    ];
    ensure!(
        source.stations.len() == ids.len(),
        "exactly five approved stops required"
    );
    for (i, station) in source.stations.iter().enumerate() {
        ensure!(
            station.id == ids[i]
                && station.map == maps[i]
                && station.constant == constants[i]
                && station.visited_flag == format!("EVENT_MN_STATION_{}_VISITED", ids[i])
                && station.menu_label == ids[i].replace('_', " ")
                && station.menu_label.len() + 2 <= 18,
            "approved station order, native identity and readable menu required"
        );
        crate::glacier_line_geometry::validate(base, station)?;
    }
    ensure!(
        base.data
            .story_event_script_constants
            .global
            .contains_key(SERVICE),
        "existing courier victory event missing"
    );
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut pack = base.clone();
    let mut added_flags = Vec::new();
    for (i, station) in source.stations.iter().enumerate() {
        let exists = pack
            .data
            .story_event_script_constants
            .global
            .contains_key(&station.visited_flag);
        if [1, 4].contains(&i) {
            ensure!(exists, "existing Staples/Winona visit event missing");
        } else {
            ensure!(!exists, "unexpected preinstalled station visit event");
            crate::northstar_lair_authoring::flag(&mut pack, &station.visited_flag)?;
            added_flags.push(station.visited_flag.clone());
        }
    }
    let flags: BTreeSet<_> = source
        .stations
        .iter()
        .map(|s| pack.data.story_event_script_constants.global[&s.visited_flag])
        .collect();
    ensure!(
        flags.len() == 5
            && !flags.contains(&pack.data.story_event_script_constants.global[SERVICE]),
        "station visits and courier authority must be distinct native events"
    );
    ensure!(
        !pack
            .data
            .initialize_events
            .event_flags
            .iter()
            .any(|f| f == SERVICE || source.stations.iter().any(|s| &s.visited_flag == f)),
        "station visits and passenger authority cannot be New Game gifts"
    );
    for (i, station) in source.stations.iter().enumerate() {
        let mut room = pack
            .data
            .maps
            .get(&station.map)
            .context("installed station")?
            .clone();
        room.attributes.fishing_group = Some("FISHGROUP_NONE".into());
        crate::glacier_line_scripts::install(&pack, &mut room, &source.stations, i)?;
        crate::authored_town_publication::replace(&mut pack, room)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    crate::glacier_line_preservation::validate(base, &pack, &source.stations, &added_flags)?;
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
