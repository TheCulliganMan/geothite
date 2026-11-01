//! Native renewable berry and apricorn stops on authored Minnesota terrain.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::grand_marais_modpack::authoring;
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-fruit-groves-v1";

#[derive(Deserialize)]
struct Groves {
    schema_version: u8,
    base_content_hash: String,
    rivers: Vec<River>,
    river_activities: Value,
    trees: Vec<Tree>,
}

#[derive(Deserialize)]
struct River {
    map: String,
    expected_blocks_sha256: String,
    blocks: Vec<u16>,
}

#[derive(Deserialize)]
struct Tree {
    id: String,
    map: String,
    tile: [u16; 2],
    approach: [u16; 2],
    item: String,
}

/// Installs harvestable native fruit trees across the authored Minnesota region.
///
/// Reinstates the accepted twenty-trainer Mississippi layouts when composing on
/// a newer regional parent, binding added river teams to the active Minnesota
/// roster. Berry-only maps retain terrain, encounters and every existing actor.
/// Each tree keeps its own saved daily harvest flag and remains visible empty.
/// The production daily reset renews fruit without resetting permanent rewards.
///
/// # Errors
/// Rejects a stale parent, river preimage/boundary mismatch, invalid regional
/// species, occupied tree identity, unsafe approach, unsupported item or pack.
pub fn build_minnesota_fruit_groves(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    build_river_content(base, source, true)
}

/// Installs the complete river terrain and activities on an exact newer parent.
///
/// Shares the verified river restoration and Minnesota roster policy with the
/// fruit publisher, but requires an empty tree list and preserves the entire
/// fruit catalog. Only the twelve wooded river modules and their activity and
/// encounter records change. Unrelated regional work and all art/audio remain.
///
/// # Errors
/// Rejects a stale parent or terrain preimage, trees in the source, duplicate
/// activities, changed water/boundaries, invalid species or an invalid pack.
pub fn build_mississippi_activity_revision(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    build_river_content(base, source, false)
}

fn build_river_content(
    base: &CompiledGamePack,
    source: &str,
    include_trees: bool,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::river_content_lineage::validate(base)?;
    let design: Groves = serde_json::from_str(source)?;
    ensure!(
        design.schema_version == 1 && design.base_content_hash == base.identity()?.content_hash,
        "fruit-grove parent differs"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m
            == if include_trees {
                MANIFEST
            } else {
                "minnesota-mississippi-fun-v1"
            }),
        "requested river/fruit stage already installed"
    );
    ensure!(
        design.rivers.len() == 12 && design.trees.is_empty() != include_trees,
        "both rivers required; fruit publication needs trees and river-only publication forbids trees"
    );
    let policy = RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )?;
    let mut pack = base.clone();
    restore_rivers(&mut pack, &design.rivers)?;
    refresh_identity(&mut pack)?;
    if !pack
        .report
        .manifests
        .iter()
        .any(|m| m == "minnesota-mississippi-fun-v1")
    {
        let mut activities = design.river_activities;
        activities["base_content_hash"] = pack.identity.content_hash.clone().into();
        for row in activities["maps"]
            .as_array_mut()
            .context("river activity maps")?
        {
            let name = row["id"].as_str().context("river activity map")?;
            row["expected_blocks_sha256"] = block_hash(&pack.data.maps[name].blocks)?.into();
            row["cells"] = json!([]);
            for activity in row["activities"]
                .as_array_mut()
                .context("river activities")?
            {
                if let Some(party) = activity["party"].as_array_mut() {
                    for member in party {
                        member["species"] = policy
                            .target(
                                &pack,
                                member["species"].as_str().context("river species")?,
                                Some(u8::try_from(
                                    member["level"].as_u64().context("river level")?,
                                )?),
                            )?
                            .into();
                    }
                }
            }
        }
        pack = crate::build_river_route_fun(&pack, &activities.to_string())?;
    }
    // The river content is being carried forward as a complete reviewed route;
    // retain its levels and rewards while updating its original four teams too.
    let river_names = design
        .rivers
        .iter()
        .map(|r| r.map.clone())
        .collect::<Vec<_>>();
    for name in &river_names {
        let map = &pack.data.maps[name];
        for object in &map.objects {
            if let Some(old) = pack.data.trainers.trainers.get(&object.script).cloned() {
                let mut trainer = old;
                for member in &mut trainer.party {
                    member.species = policy.target(&pack, &member.species, Some(member.level))?;
                    member.moves = pack
                        .data
                        .create_pokemon(&member.species, member.level, member.dvs)?
                        .moves;
                }
                pack.data
                    .trainers
                    .trainers
                    .insert(object.script.clone(), trainer);
            }
        }
        if let Some(encounters) = pack.data.wild_encounters.get(name) {
            let mut value = serde_json::to_value(encounters)?;
            regional_encounters(&pack, &policy, &mut value)?;
            pack.data
                .wild_encounters
                .insert(name.clone(), serde_json::from_value(value)?);
        }
    }
    let template = base
        .data
        .maps
        .values()
        .flat_map(|m| &m.objects)
        .find(|o| o.sprite == "SPRITE_FRUIT_TREE")
        .context("native fruit-tree object/art")?
        .clone();
    let mut maps = BTreeMap::<String, MapModule>::new();
    for tree in &design.trees {
        ensure!(
            tree.id.starts_with("FRUITTREE_MN_") && !pack.data.fruit_trees.0.contains_key(&tree.id),
            "occupied fruit tree identity"
        );
        ensure!(
            matches!(
                tree.item.as_str(),
                "BERRY"
                    | "GOLD_BERRY"
                    | "PSNCUREBERRY"
                    | "PRZCUREBERRY"
                    | "BITTER_BERRY"
                    | "MINT_BERRY"
                    | "ICE_BERRY"
                    | "BURNT_BERRY"
                    | "MYSTERYBERRY"
                    | "RED_APRICORN"
                    | "BLU_APRICORN"
                    | "YLW_APRICORN"
                    | "GRN_APRICORN"
                    | "WHT_APRICORN"
                    | "BLK_APRICORN"
                    | "PNK_APRICORN"
            ) && pack.data.items.contains_key(&tree.item),
            "native berry/apricorn item required"
        );
        let mut map = maps
            .remove(&tree.map)
            .or_else(|| pack.data.maps.get(&tree.map).cloned())
            .context("tree map")?;
        validate_tree(&pack, &map, tree)?;
        let script = format!("{}Script", tree.id);
        ensure!(!map.scripts.contains_key(&script), "occupied fruit script");
        let body = json!([authoring::command("fruittree", &[&tree.id])]);
        let mut object = template.clone();
        object.x = tree.tile[0];
        object.y = tree.tile[1];
        object.script = script.clone();
        object.object_identifier = Some(tree.id.clone());
        object.label = None;
        object.event_flag = "-1".into();
        map.objects.push(object);
        map.scripts.insert(script.clone(), body.clone());
        let pickups = crate::parse_script_field_pickups(
            &map.id,
            &BTreeMap::from([(script, body)]),
            &map.objects,
        )?;
        ensure!(
            pickups.len() == 1 && pickups[0].fruit_tree_id.as_deref() == Some(&tree.id),
            "native fruit script lowering failed"
        );
        map.script_field_pickups.extend(pickups);
        authoring::refresh_event_section(&mut map);
        pack.data
            .fruit_trees
            .0
            .insert(tree.id.clone(), tree.item.clone());
        maps.insert(tree.map.clone(), map);
    }
    for map in maps.into_values() {
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_manifest == base.audio_manifest,
        "unrelated art/audio changed"
    );
    if include_trees {
        pack.report.manifests.push(MANIFEST.into());
    }
    refresh_identity(&mut pack)?;
    crate::river_content_lineage::validate(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn refresh_identity(pack: &mut CompiledGamePack) -> Result<()> {
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    Ok(())
}

fn block_hash(blocks: &[u16]) -> Result<String> {
    let bytes = blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn restore_rivers(pack: &mut CompiledGamePack, rivers: &[River]) -> Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for river in rivers {
        ensure!(
            names.insert(&river.map)
                && (river.map.starts_with("MnMississippiLowerWoodsC")
                    || river.map.starts_with("MnMississippiUpstreamWoodsC")),
            "exact distinct wooded river sources required"
        );
        let old = pack
            .data
            .maps
            .get(&river.map)
            .context("existing river map")?;
        ensure!(
            old.attributes.tileset_name == "johto"
                && block_hash(&old.blocks)? == river.expected_blocks_sha256
                && old.blocks.len() == river.blocks.len(),
            "river preimage differs: {}",
            river.map
        );
        let mut map = old.clone();
        map.blocks = river.blocks.clone();
        let col = if river.map.contains("C00") { 0 } else { 1 };
        let row = if river.map.ends_with("R00") {
            0
        } else if river.map.ends_with("R01") {
            1
        } else {
            2
        };
        for (i, (&a, &b)) in old.blocks.iter().zip(&map.blocks).enumerate() {
            let x = col * 32 + i % 32;
            let y = row * 24 + i / 32;
            if x == 0 || x == 63 || y == 0 || y == 67 {
                ensure!(a == b, "river mouth/exterior boundary changed");
            }
        }
        for y in 0..old.attributes.height * 2 {
            for x in 0..old.attributes.width * 2 {
                if crate::authored_town_geometry::kind(pack, old, [x, y])? == "WATER" {
                    ensure!(
                        crate::authored_town_geometry::kind(pack, &map, [x, y])? == "WATER",
                        "river restoration changed original water"
                    );
                }
            }
        }
        crate::authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}

fn validate_tree(pack: &CompiledGamePack, map: &MapModule, tree: &Tree) -> Result<()> {
    ensure!(
        map.id.starts_with("Mn")
            && matches!(
                map.attributes.environment.as_deref(),
                Some("TOWN" | "ROUTE" | "OUTDOOR")
            ),
        "Minnesota exterior required"
    );
    let [x, y] = tree.tile;
    ensure!(
        x >= 3 && y >= 3 && x + 3 < map.attributes.width * 2 && y + 3 < map.attributes.height * 2,
        "fruit tree too close to map seam"
    );
    ensure!(
        tree.tile[0].abs_diff(tree.approach[0]) + tree.tile[1].abs_diff(tree.approach[1]) == 1,
        "adjacent harvest approach required"
    );
    ensure!(
        crate::authored_town_geometry::kind(pack, map, tree.tile)? == "FLOOR",
        "fruit tree needs native Floor"
    );
    for dy in -1i16..=1 {
        for dx in -1i16..=1 {
            let q = [
                u16::try_from(i16::try_from(x)? + dx)?,
                u16::try_from(i16::try_from(y)? + dy)?,
            ];
            ensure!(
                matches!(
                    crate::authored_town_geometry::kind(pack, map, q)?,
                    "FLOOR" | "TALL_GRASS"
                ) && !map.objects.iter().any(|o| [o.x, o.y] == q),
                "tree needs a clear walking ring"
            );
        }
    }
    ensure!(
        !map.events
            .warps
            .iter()
            .any(|w| w.x.abs_diff(x) + w.y.abs_diff(y) < 3)
            && !map
                .events
                .coord_events
                .iter()
                .any(|e| e.x.abs_diff(x) + e.y.abs_diff(y) < 3)
            && !map
                .events
                .bg_events
                .iter()
                .any(|e| e.x.abs_diff(x) + e.y.abs_diff(y) < 3),
        "tree obstructs a native event approach"
    );
    Ok(())
}

fn regional_encounters(
    pack: &CompiledGamePack,
    policy: &RegionalRoster,
    value: &mut Value,
) -> Result<()> {
    match value {
        Value::Object(fields) => {
            if let Some(name) = fields.get("species").and_then(Value::as_str) {
                let level = fields
                    .get("level")
                    .and_then(Value::as_u64)
                    .map(u8::try_from)
                    .transpose()?;
                fields.insert("species".into(), policy.target(pack, name, level)?.into());
            }
            for child in fields.values_mut() {
                regional_encounters(pack, policy, child)?;
            }
        }
        Value::Array(rows) => {
            for row in rows {
                regional_encounters(pack, policy, row)?;
            }
        }
        _ => {}
    }
    Ok(())
}
