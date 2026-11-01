//! Connected lake-country chapter, preserving the existing wind/water gyms.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const BASE: &str = "58a1dc84c01d3c5d7d0c5efb849a024a5c6ece1daadb525a69055a32c55b4047";
const MANIFEST: &str = "minnesota-lake-country-story-v1";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Chapter {
    schema_version: u8,
    manifest: String,
    base_content_hash: String,
    links: Vec<Link>,
    caps: Vec<crate::lake_country_geometry::Cap>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Link {
    town: String,
    route: String,
    direction: String,
    offset: i32,
    mouth: [u16; 2],
    town_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
}

/// Connects Minneapolis/R05/St Cloud/R06/Brainerd without granting progression.
///
/// Current and Capitol permit northward departure; courier/manifest completion
/// is independent. Wind permits leaving St Cloud for R06. Return travel is free.
/// Existing gyms, battles, source art, audio and recovery stay exact. The ranger
/// retains his success-only Surf/report logic, with canonical MS05 testimony.
/// Unbound Brainerd branches and stray R06 lanes receive visible forest caps.
///
/// # Errors
/// Returns an error for stale/duplicate inputs, altered gates or anchors,
/// unsafe walking/Surf boundaries, incorrect native connections or failed
/// script/storage/runtime validation. The borrowed input stays unchanged.
pub fn build_lake_country_story_modpack(
    base: &CompiledGamePack,
    chapter_json: &str,
    story_json: &str,
    r05_json: &str,
    r06_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let chapter: Chapter = serde_json::from_str(chapter_json)?;
    ensure!(
        chapter.schema_version == 1
            && chapter.manifest == MANIFEST
            && chapter.base_content_hash == BASE
            && base.identity.content_hash == BASE
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact courier story required; duplicate/stale lake-country chapter"
    );
    let r05: Value = serde_json::from_str(r05_json)?;
    let r06: Value = serde_json::from_str(r06_json)?;
    ensure!(
        r05["id"] == "MN_R05"
            && r05["required_flags"] == json!(["BADGE_3", "BADGE_4"])
            && r06["id"] == "MN_R06"
            && r06["required_flags"] == json!(["BADGE_5"]),
        "canonical two-city/Wind road gates required; no courier gate"
    );
    let mut maps = [
        "MnMinneapolis",
        "MnR05C00R02",
        "MnStCloud",
        "MnR06C00R00",
        "MnR06C00R01",
        "MnBrainerd",
    ]
    .into_iter()
    .map(|id| {
        Ok((
            id.to_owned(),
            base.data
                .maps
                .get(id)
                .context("installed lake-country map")?
                .clone(),
        ))
    })
    .collect::<Result<BTreeMap<String, MapModule>>>()?;
    crate::lake_country_geometry::cap(base, &mut maps, &chapter.caps)?;
    ensure!(
        chapter.links.len() == 2,
        "both lake-country city links required"
    );
    for (link, (town, road, direction, offset, mouth, town_edge, road_edge)) in
        chapter.links.iter().zip([
            (
                "MnMinneapolis",
                "MnR05C00R02",
                "north",
                5,
                [16, 0],
                [[32, 0], [33, 0]],
                [[22, 15], [23, 15]],
            ),
            (
                "MnBrainerd",
                "MnR06C00R00",
                "south",
                -5,
                [16, 23],
                [[32, 47], [33, 47]],
                [[42, 0], [43, 0]],
            ),
        ])
    {
        ensure!(
            link.town == town
                && link.route == road
                && link.direction == direction
                && link.offset == offset
                && link.mouth == mouth
                && link.town_edge == town_edge
                && link.route_edge == road_edge,
            "fixed native lake-country mouths required"
        );
        let city = maps.get_mut(town).context("city mouth")?;
        let index =
            usize::from(mouth[1]) * usize::from(city.attributes.width) + usize::from(mouth[0]);
        ensure!(
            city.blocks[index] == if town == "MnMinneapolis" { 5 } else { 1 },
            "stale city mouth"
        );
        city.blocks[index] = 1;
        let reverse = if direction == "north" {
            "south"
        } else {
            "north"
        };
        crate::lake_country_geometry::connect(city, direction, road, offset)?;
        crate::lake_country_geometry::connect(
            maps.get_mut(road).context("road mouth")?,
            reverse,
            town,
            -offset,
        )?;
    }
    crate::lake_country_checkpoints::install(base, &mut maps)?;
    let story: Value = serde_json::from_str(story_json)?;
    let milestones = story["milestones"].as_array().context("story milestones")?;
    let ms = milestones
        .iter()
        .filter(|m| m["id"] == "MS05")
        .collect::<Vec<_>>();
    ensure!(
        ms.len() == 1
            && ms[0]["location"] == "MnBrainerd"
            && ms[0]["requires"] == json!(["BADGE_6"])
            && ms[0]["sets"] == json!(["EVENT_MN_BRAINERD_REPORT", "EVENT_MN_SURF_GIFT_RECEIVED"]),
        "canonical optional ranger report/Surf contract required"
    );
    let pages: Vec<Vec<String>> = serde_json::from_value(ms[0]["dialogue_pages"].clone())?;
    ensure!(
        pages.len() == 2
            && pages.iter().all(|p| p.len() == 2
                && p.iter()
                    .all(|s| !s.is_empty() && s.is_ascii() && s.len() <= 18)),
        "two native MS05 pages required"
    );
    let brainerd = maps.get_mut("MnBrainerd").context("ranger")?;
    ensure!(
        brainerd.scripts.contains_key("MN_BRAINERD_RANGERText"),
        "installed ranger report missing"
    );
    authoring::text(
        brainerd,
        "MN_BRAINERD_RANGERText",
        &pages
            .iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    // The northern branch is visibly closed until its connection is published.
    // Keep the gift useful without promising an unbound road.
    for (label, lines) in [
        (
            "MnBrainerdRangerGiftText",
            ["SURF is yours.", "Explore the lakes."],
        ),
        (
            "MnBrainerdRangerRepeatText",
            ["Their testimony", "belongs on record."],
        ),
    ] {
        ensure!(
            brainerd.scripts.contains_key(label),
            "installed ranger gift/repeat text missing"
        );
        authoring::text(brainerd, label, &lines);
    }
    crate::lake_country_geometry::validate(base, &maps)?;
    let mut pack = base.clone();
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect();
    for (_, mut map) in maps {
        authoring::finish(&pack, &mut map)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_variable_commands =
            crate::parse_script_variable_commands(&map.id, &map.scripts)?;
        map.script_movements =
            crate::parse_script_movements(&map.id, &map.scripts, &map.script_object_commands)?;
        map.script_map_commands = crate::parse_script_map_commands(&map.id, &map.scripts, &names)?;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    crate::lake_country_geometry::verify(&pack)?;
    pack.report.manifests.push(MANIFEST.into());
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
