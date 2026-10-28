//! Grassy riverbank expeditions with native trainers, supplies and field care.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_route_design::{Pickup, RouteTrainer};
use crate::grand_marais_modpack::authoring;

const MANIFEST: &str = "minnesota-mississippi-fun-v1";

#[derive(Deserialize)]
struct Revision {
    schema_version: u16,
    base_content_hash: String,
    maps: Vec<BankMap>,
}

#[derive(Deserialize)]
struct BankMap {
    id: String,
    expected_blocks_sha256: String,
    cells: Vec<Cell>,
    activities: Vec<Value>,
}

#[derive(Deserialize)]
struct Cell {
    metatile: [u16; 2],
    before: u16,
    after: u16,
}

/// Adds eighteen trainers, grass walks, three supplies and a medic to each reach.
///
/// Retains the four original bank trainers, all river mouths, water quadrants,
/// native connections, encounter tables and all unrelated game/audio content.
/// Each reach has twenty trainers in total. New pickups and battle rewards use
/// native once-only flags; installation grants no progress or resources.
///
/// # Errors
/// Rejects a stale parent, duplicate installation, unsupported terrain/activity,
/// changed external boundary, erased water, occupied actor or invalid pack.
pub fn build_river_route_fun(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let revision: Revision = serde_json::from_str(source)?;
    ensure!(
        revision.schema_version == 1 && revision.maps.len() == 12,
        "twelve wooded river chunks required"
    );
    ensure!(
        base.identity()?.content_hash == revision.base_content_hash,
        "river fun parent differs"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "river activities already installed"
    );
    let mut pack = base.clone();
    let mut unique = BTreeSet::new();
    let mut flags = Vec::new();
    for bank in &revision.maps {
        ensure!(
            unique.insert(&bank.id)
                && (bank.id.starts_with("MnMississippiLowerWoodsC")
                    || bank.id.starts_with("MnMississippiUpstreamWoodsC")),
            "unsupported or duplicate river map"
        );
        for activity in &bank.activities {
            let id = activity["id"].as_str().context("river activity id")?;
            ensure!(
                id.starts_with("MN_RIVER_FUN_")
                    && !base.data.maps.values().any(|m| m.scripts.contains_key(id)),
                "occupied activity identity"
            );
            if let Some(flag) = activity["defeated_flag"]
                .as_str()
                .or(activity["flag"].as_str())
            {
                flags.push(flag.to_owned());
            }
        }
    }
    ensure!(
        flags.len() == 42 && flags.iter().collect::<BTreeSet<_>>().len() == 42,
        "36 trainers and six unique pickups required"
    );
    crate::authored_city_expansion_events::flags(&mut pack, &flags)?;
    let names = pack
        .data
        .maps
        .iter()
        .filter_map(|(n, m)| {
            m.attributes
                .map_constant
                .as_ref()
                .map(|c| (c.clone(), n.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    for bank in &revision.maps {
        let original = base
            .data
            .maps
            .get(&bank.id)
            .context("installed river chunk")?;
        let mut map = original.clone();
        let bytes = original
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            format!("{:x}", Sha256::digest(bytes)) == bank.expected_blocks_sha256
                && map.attributes.tileset_name == "johto",
            "river terrain preimage differs: {}",
            bank.id
        );
        let collision = &base.data.tilesets["johto"].collision;
        let mut cells = BTreeSet::new();
        for cell in &bank.cells {
            let [x, y] = cell.metatile;
            ensure!(
                x < map.attributes.width && y < map.attributes.height && cells.insert([x, y]),
                "invalid river cell"
            );
            // Internal chunk edges may gain grass. The entire outer rectangle
            // remains fixed, so every regional mouth and closed bank survives.
            let col = if bank.id.contains("C00") { 0 } else { 1 };
            let row = if bank.id.ends_with("R00") {
                0
            } else if bank.id.ends_with("R01") {
                1
            } else {
                2
            };
            let [gx, gy] = [col * 32 + x, row * 24 + y];
            ensure!(
                gx > 0 && gx < 63 && gy > 0 && gy < 67,
                "external river boundary changed"
            );
            let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
            ensure!(
                map.blocks[index] == cell.before && matches!(cell.after, 2 | 3 | 0x35),
                "unsupported river terrain edit"
            );
            let before = collision
                .get(&format!("{:02x}", cell.before))
                .context("original river collision")?;
            let after = collision
                .get(&format!("{:02x}", cell.after))
                .context("new river collision")?;
            ensure!(
                before
                    .iter()
                    .zip(after)
                    .all(|(a, b)| a != "WATER" || b == "WATER"),
                "river water erased"
            );
            map.blocks[index] = cell.after;
        }
        let mut occupied = map
            .objects
            .iter()
            .map(|o| [o.x, o.y])
            .collect::<BTreeSet<_>>();
        for activity in &bank.activities {
            let tile: [u16; 2] = serde_json::from_value(activity["tile"].clone())?;
            ensure!(
                occupied.insert(tile)
                    && matches!(
                        crate::authored_town_geometry::kind(&pack, &map, tile)?,
                        "FLOOR" | "TALL_GRASS"
                    ),
                "river activity needs unoccupied walkable ground"
            );
            match activity["kind"].as_str().context("activity kind")? {
                "trainer" => {
                    let trainer: RouteTrainer = serde_json::from_value(activity.clone())?;
                    let global = &mut pack.data.story_event_script_constants.global;
                    let next = pack
                        .data
                        .trainers
                        .trainers
                        .values()
                        .filter(|t| t.trainer_class == trainer.trainer_class)
                        .filter_map(|t| global.get(&t.trainer_id))
                        .max()
                        .context("existing trainer namespace")?
                        .checked_add(1)
                        .context("trainer id overflow")?;
                    ensure!(
                        next <= 255 && !global.contains_key(&trainer.id),
                        "occupied native trainer identity"
                    );
                    global.insert(trainer.id.clone(), next);
                    crate::authored_city_expansion_events::activities(
                        &mut pack,
                        &mut map,
                        &[trainer],
                        &[],
                    )?;
                }
                "item" => {
                    let item: Pickup = serde_json::from_value(activity.clone())?;
                    crate::authored_city_expansion_events::activities(
                        &mut pack,
                        &mut map,
                        &[],
                        &[item],
                    )?;
                }
                "healer" => {
                    let id = activity["id"].as_str().context("medic id")?;
                    let sprite = activity["sprite"].as_str().context("medic sprite")?;
                    let label = format!("{id}Text");
                    let lines: Vec<String> = serde_json::from_value(activity["lines"].clone())?;
                    authoring::text(
                        &mut map,
                        &label,
                        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
                    );
                    let c = authoring::command;
                    map.scripts.insert(
                        id.into(),
                        json!([
                            c("faceplayer", &[]),
                            c("opentext", &[]),
                            c("writetext", &[&label]),
                            c("waitbutton", &[]),
                            c("special", &["HealParty"]),
                            c("closetext", &[]),
                            c("end", &[])
                        ]),
                    );
                    authoring::npc(
                        &pack,
                        &mut map,
                        id,
                        sprite,
                        i16::try_from(tile[0])?,
                        i16::try_from(tile[1])?,
                    )?;
                    map.objects
                        .last_mut()
                        .context("medic object")?
                        .object_identifier = Some(id.into());
                }
                _ => anyhow::bail!("unsupported river activity"),
            }
        }
        crate::authored_city_expansion_events::finish(&pack, &mut map, &names)?;
        ensure!(
            map.attributes == original.attributes
                && map.events == original.events
                && map.objects.starts_with(&original.objects)
                && original
                    .scripts
                    .iter()
                    .all(|(k, v)| map.scripts.get(k) == Some(v)),
            "original river contracts changed"
        );
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    for prefix in ["MnMississippiLowerWoodsC", "MnMississippiUpstreamWoodsC"] {
        let count = pack
            .data
            .maps
            .values()
            .filter(|m| m.id.starts_with(prefix))
            .flat_map(|m| &m.objects)
            .filter(|o| o.object_type == "OBJECTTYPE_TRAINER")
            .count();
        ensure!(
            count == 20,
            "each river reach needs twenty trainers, found {count}"
        );
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.data.wild_encounters == base.data.wild_encounters,
        "unrelated art/audio/encounters changed"
    );
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
