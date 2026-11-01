//! Native working crops and a transactional Berry gift beside the R02 farmhouse.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnR02C00R00";
const ACTOR: &str = "MN_R02_GARDEN_GROWER";
const FLAG: &str = "EVENT_MN_R02_GARDEN_BERRY";
const SOURCE: &str = "1348ff5a00f57b5b2add40945585f68fdf4c1e4c2cda638e840c4be1cf4cc875";
const PARENT: &str = "c9c4af424190022ac77a1588010edc8edade235ea9fad5f6973f2a93a2e25d7d";
const MARKER: &str = "minnesota-r02-working-farmyard-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    parent_content_hash: String,
    native_module_sha256: String,
    source_pixels_modified: bool,
    before_rows: Vec<String>,
    rows: Vec<String>,
    changed_metatiles: usize,
    new_actor: Actor,
    reward: Reward,
}

#[derive(Deserialize)]
struct Actor {
    id: String,
    sprite: String,
    tile: [u16; 2],
    approach: [u16; 2],
}

#[derive(Deserialize)]
struct Reward {
    item: String,
    quantity: u8,
    event_flag: String,
    requires_badge: bool,
    requires_field_move: bool,
    text: Vec<String>,
    full_text: Vec<String>,
    received_text: Vec<String>,
    repeat_text: Vec<String>,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn blocks(rows: &[String]) -> Result<Vec<u16>> {
    ensure!(rows.len() == 22, "complete farm rows required");
    let mut blocks = Vec::with_capacity(660);
    for row in rows {
        let row = row
            .split_whitespace()
            .map(|b| u8::from_str_radix(b, 16).map(u16::from))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(row.len() == 30, "complete farm columns required");
        blocks.extend(row);
    }
    Ok(blocks)
}

fn add_grower(pack: &CompiledGamePack, map: &mut MapModule, plan: &Proposal) -> Result<()> {
    let constant = map
        .attributes
        .map_constant
        .as_deref()
        .context("farm map constant")?;
    // Compile only the new program. Recompiling the original whole module could
    // silently normalize unrelated authored commands from an earlier epoch.
    let mut extra = authoring::blank(pack, MAP, MAP, constant)?;
    northstar_lair_authoring::actor(
        pack,
        &mut extra,
        ACTOR,
        &plan.new_actor.sprite,
        plan.new_actor.tile,
    )?;
    let reward = &plan.reward;
    for (label, lines) in [
        ("MnR02GardenOfferText", &reward.text),
        ("MnR02GardenFullText", &reward.full_text),
        ("MnR02GardenReceivedText", &reward.received_text),
        ("MnR02GardenRepeatText", &reward.repeat_text),
    ] {
        ensure!(!lines.is_empty(), "nonempty grower text required");
        authoring::text(
            &mut extra,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    extra.scripts.insert(
        ACTOR.into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[FLAG]),
            c("iftrue", &["MnR02GardenRepeat"]),
            c("writetext", &["MnR02GardenOfferText"]),
            c("waitbutton", &[]),
            c("giveitem", &["BERRY", "1"]),
            c("iffalse", &["MnR02GardenFull"]),
            c("setevent", &[FLAG]),
            c("writetext", &["MnR02GardenReceivedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for (label, text) in [
        ("MnR02GardenFull", "MnR02GardenFullText"),
        ("MnR02GardenRepeat", "MnR02GardenRepeatText"),
    ] {
        extra.scripts.insert(
            label.into(),
            json!([
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    ensure!(
        extra
            .scripts
            .keys()
            .all(|label| !pack.data.map_scripts.contains_key(label)
                && !map.scripts.contains_key(label)),
        "grower labels already occupied"
    );
    let names = pack.data.map_name_by_constant_from_attributes()?;
    crate::authored_city_expansion_events::finish(pack, &mut extra, &names)?;
    map.objects.extend(extra.objects);
    map.scripts.extend(extra.scripts);
    map.script_flag_commands.extend(extra.script_flag_commands);
    map.script_control_commands
        .extend(extra.script_control_commands);
    map.script_text_commands.extend(extra.script_text_commands);
    map.script_text_bodies.extend(extra.script_text_bodies);
    map.script_item_grants.extend(extra.script_item_grants);
    map.script_object_commands
        .extend(extra.script_object_commands);
    map.script_runtime_commands
        .extend(extra.script_runtime_commands);
    map.script_variable_commands
        .extend(extra.script_variable_commands);
    map.script_map_commands.extend(extra.script_map_commands);
    authoring::refresh_event_section(map);
    Ok(())
}

/// Fits the reviewed working garden and once-only native Berry gift on R02.
///
/// Preserves both roads, all water and bridge collision, the original bank hop,
/// both buildings/rooms, all existing actors/programs, current regional fauna,
/// art and audio. Adds one always-present grower and one persistent claim flag;
/// capacity refusal cannot consume the claim. Returns an unselected candidate.
///
/// # Errors
/// Rejects source/parent/module drift, occupied new identities, invalid terrain,
/// blocked approaches, changed original data or failed strict pack verification.
/// The borrowed parent and source remain unchanged on failure.
pub fn revise_r02_working_farmyard(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed farmyard source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.map == MAP
            && plan.tileset == "mn_prairie_kanto"
            && plan.dimensions_metatiles == [30, 22]
            && !plan.source_pixels_modified
            && plan.parent_content_hash == PARENT
            && base.identity.content_hash == PARENT
            && plan.changed_metatiles == 17
            && plan.new_actor.id == ACTOR
            && plan.new_actor.sprite == "SPRITE_GRANNY"
            && plan.new_actor.tile == [9, 19]
            && plan.new_actor.approach == [10, 19]
            && plan.reward.item == "BERRY"
            && plan.reward.quantity == 1
            && plan.reward.event_flag == FLAG
            && !plan.reward.requires_badge
            && !plan.reward.requires_field_move
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "current complete farm parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("current farm road")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.native_module_sha256
            && original.blocks == blocks(&plan.before_rows)?
            && original.attributes.tileset_name == plan.tileset
            && [original.attributes.width, original.attributes.height] == [30, 22]
            && original.objects.len() == 7
            && original.events.warps.len() == 4,
        "complete farm preimage differs"
    );
    let mut map = original.clone();
    map.blocks = blocks(&plan.rows)?;
    let changes = map
        .blocks
        .iter()
        .zip(&original.blocks)
        .enumerate()
        .filter_map(|(i, (a, b))| (a != b).then_some(i))
        .collect::<Vec<_>>();
    ensure!(
        changes.len() == 17
            && changes
                .iter()
                .all(|i| (3..=9).contains(&(i % 30)) && (4..=10).contains(&(i / 30))),
        "protected farm terrain changed"
    );
    for p in [plan.new_actor.tile, plan.new_actor.approach] {
        ensure!(
            kind(base, &map, p)? == "FLOOR",
            "grower must have dry footing"
        );
    }
    for y in 12..=19 {
        ensure!(
            kind(base, &map, [12, y])? == "FLOOR" && kind(base, &map, [13, y])? == "FLOOR",
            "two farm approach lanes required"
        );
    }
    let mut child = base.clone();
    northstar_lair_authoring::flag(&mut child, FLAG)?;
    add_grower(&child, &mut map, &plan)?;
    crate::authored_town_publication::replace(&mut child, map)?;
    let attributes = base
        .data
        .map_attributes
        .get(MAP)
        .context("original farm attribute table")?;
    child
        .data
        .map_attributes
        .insert(MAP.into(), attributes.clone());
    child.report.manifests.push(MARKER.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    crate::validate_compact_superior_geography(&child)?;
    let mut residual = child.clone();
    crate::authored_town_publication::replace(&mut residual, original.clone())?;
    residual
        .data
        .map_attributes
        .insert(MAP.into(), attributes.clone());
    residual
        .data
        .story_event_script_constants
        .global
        .remove(FLAG);
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(residual == *base, "unrelated farmyard content changed");
    Ok(child)
}
