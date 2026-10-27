//! Independent headwaters reading, acknowledged comparison and Iron Range gate.
use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_publication::{CityIdentity, publish};
use crate::grand_marais_modpack::authoring;

const BASE: &str = "43b3d65bda72a8904cfad17781cc0da408045c64d91ee22fb5c6b3b1634006db";
const MANIFEST: &str = "minnesota-itasca-story-v1";

/// Publishes Itasca and connects the independent reading/comparison chapter.
///
/// Lake permits the public R14 outing, including an early visit before Pine.
/// Pine permits recording the marker. Separately acknowledging the officer's
/// comparison records the report; only Pine plus that report permits R09.
/// Inbound returns are free. No item, badge, money or healing is granted.
/// Original art, audio, encounters, HOME/recovery and other story stay intact.
///
/// # Errors
/// Returns an error for stale/duplicate input, conflicting native identities,
/// changed prerequisites, unsafe geometry, invalid script lowering, broken
/// native connections or failed pack validation. The input remains unchanged.
pub fn build_itasca_story_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    story_json: &str,
    r14_json: &str,
    r09_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity.content_hash == BASE && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact shore input required; duplicate/stale Itasca publication"
    );
    let town: Town = serde_json::from_str(town_json)?;
    let raw: Value = serde_json::from_str(town_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnLakeItasca"
            && town.native_constant == "MN_LAKE_ITASCA"
            && town.dimensions_metatiles == [24, 24]
            && town.compiled_tileset == "mn_headwaters"
            && town.interiors.len() == 1
            && town.doors.is_empty()
            && !base.data.maps.contains_key(&town.id)
            && layout.id == town.id
            && layout.schema_version == 2
            && layout.dimensions_metatiles == [24, 24]
            && layout.compiled_tileset == town.compiled_tileset,
        "complete Itasca reference site with native side-entry gatehouse required"
    );
    for (source, name, flags) in [
        (r14_json, "MN_R14", json!(["BADGE_6"])),
        (
            r09_json,
            "MN_R09",
            json!(["BADGE_7", "EVENT_MN_HEADWATERS_REPORT"]),
        ),
    ] {
        let route: Value = serde_json::from_str(source)?;
        ensure!(
            route["id"] == name && route["required_flags"] == flags,
            "canonical reference/report road conditions required"
        );
    }
    let mut pack = base.clone();
    crate::itasca_events::allocate(&mut pack)?;
    let mut map = authoring::blank(&pack, "Route30", &town.id, &town.native_constant)?;
    map.attributes.width = 24;
    map.attributes.height = 24;
    map.attributes.border_block = 5;
    map.attributes.tileset_name = town.compiled_tileset.clone();
    map.blocks = layout.compile()?;
    crate::authored_town_scripts::ordinary(&pack, &mut map, &town.residents, &town.signs, "", "")?;
    crate::itasca_geometry::validate(&pack, &map, &raw)?;
    crate::itasca_events::marker(&mut map, &serde_json::from_str(story_json)?)?;
    crate::authored_town_scripts::finish(&pack, &mut map, &town)?;
    let mut gate = crate::itasca_gatehouse::build(&mut pack, &town, &raw, &mut map)?;
    crate::authored_town_scripts::finish(&pack, &mut map, &town)?;
    crate::authored_town_scripts::finish(&pack, &mut gate, &town)?;
    crate::authored_town_geometry::validate_with_edges(
        &pack,
        &town,
        &[(map.clone(), [42, 24])],
        &BTreeSet::new(),
    )?;
    // A public landmark has no new clinic, HOME or blackout destination.
    publish(
        &mut pack,
        vec![(map, [42, 24]), (gate, [1, 4])],
        CityIdentity {
            group: "MINNESOTA_ITASCA",
            group_id: 79,
            landmark: "LANDMARK_MN_LAKE_ITASCA",
            fallback_landmark: Some("LANDMARK_MN_HEADWATERS"),
            constant: "MN_LAKE_ITASCA",
            name: "LAKE ITASCA",
            position: [48, 64],
            manifest: MANIFEST,
            town: "",
            clinic: "",
            gym: "",
        },
    )?;
    crate::itasca_connections::install(&mut pack, &town, &serde_json::from_str(story_json)?)?;
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
