//! Fixed native badge-controlled buoys for the Minneapolis north river pool.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{CoordEvent, MapScriptSectionCommand};
use serde_json::json;

use crate::authored_city_expansion::merge_new_content;
use crate::authored_city_expansion_events as events;
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const CHECK: &str = "MnMPUpperPoolBuoyCheck";
const REFRESH: &str = "MnMPUpperPoolBuoyRefresh";
const OPEN: &str = "MnMPUpperPoolBuoyOpen";
const DENIED: &str = "MnMPUpperPoolBuoyDenied";
const DONE: &str = "MnMPUpperPoolBuoyDone";
const TEXT: &str = "MnMPUpperPoolBuoyDeniedText";
const CELLS: [(u16, u16); 3] = [(16, 88), (17, 53), (18, 89)];

/// Adds the fixed north-pool gate to an already recomposed Minneapolis module.
///
/// The caller supplies a closed base layout: native block49 at metas16..18,1.
/// Only both actual badge checks reopen the exact native shoreline88/53/89.
/// A silent tile callback reopens authorized water after Continue, submenu and
/// map setup; denial never moves the player. Existing parsed program tables are
/// extended without reparsing or changing any old program.
///
/// # Errors
/// Returns an error for another city/extent/alias, invalid native collisions,
/// a changed closed recipe, missing old coordinate events, occupied approaches,
/// script-name collisions or malformed new programs. The city is unchanged on
/// failure. The caller owns normal split-payload publication and verification.
pub(super) fn install(pack: &CompiledGamePack, city: &mut MapModule) -> Result<()> {
    validate(pack, city)?;
    let mut additions = authoring::blank(pack, &city.id, &city.id, "MN_MINNEAPOLIS")?;
    additions.attributes = city.attributes.clone();
    additions.blocks = city.blocks.clone();
    let c = authoring::command;
    for (entry, failure) in [(CHECK, DENIED), (REFRESH, DONE)] {
        additions.scripts.insert(
            entry.into(),
            json!([
                c("checkflag", &["ENGINE_PLAINBADGE"]),
                c("iffalse", &[failure]),
                c("checkflag", &["ENGINE_FOGBADGE"]),
                c("iffalse", &[failure]),
                c("sjump", &[OPEN])
            ]),
        );
    }
    additions.scripts.insert(
        OPEN.into(),
        json!([
            c("changeblock", &["32", "2", "88"]),
            c("changeblock", &["34", "2", "53"]),
            c("changeblock", &["36", "2", "89"]),
            c("end", &[])
        ]),
    );
    additions
        .scripts
        .insert(DONE.into(), json!([c("end", &[])]));
    additions.scripts.insert(
        DENIED.into(),
        json!([
            c("opentext", &[]),
            c("writetext", &[TEXT]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    authoring::text(
        &mut additions,
        TEXT,
        &["River gate closed.", "Earn CURRENT and", "CAPITOL BADGES."],
    );
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|metadata| (metadata.constant.clone(), metadata.name.clone()))
        .collect::<BTreeMap<_, _>>();
    events::finish(pack, &mut additions, &names)?;
    // The shared new-activity finisher does not parse changeblock commands.
    additions.script_block_changes =
        crate::parse_script_block_changes(&additions.id, &additions.scripts)?;
    ensure!(
        additions.script_block_changes.len() == 3,
        "north river gate needs exactly three native block writes"
    );
    let mut revised = city.clone();
    merge_new_content(&mut revised, additions)?;
    for y in [1, 4] {
        for x in 32..=37 {
            revised.events.coord_events.push(CoordEvent {
                x,
                y,
                scene_id: "-1".into(),
                script_name: CHECK.into(),
            });
        }
    }
    revised
        .map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec!["MAPCALLBACK_TILES".into(), REFRESH.into()],
            command_index: revised.map_script_section_commands.len(),
        });
    authoring::refresh_event_section(&mut revised);
    *city = revised;
    Ok(())
}

fn validate(pack: &CompiledGamePack, city: &MapModule) -> Result<()> {
    ensure!(
        city.id == "MnMinneapolis"
            && city.attributes.map_constant.as_deref() == Some("MN_MINNEAPOLIS")
            && city.attributes.width == 40
            && city.attributes.height == 32
            && city.attributes.tileset_name == "mn_minneapolis_city"
            && city.blocks.len() == 40 * 32,
        "north river checkpoint requires compact40x32 Minneapolis"
    );
    let collision = &pack
        .data
        .tilesets
        .get("mn_minneapolis_city")
        .context("installed Minneapolis native art")?
        .collision;
    ensure!(
        collision.get("31").is_some_and(|quadrants| {
            quadrants
                .iter()
                .map(String::as_str)
                .eq(["BUOY", "BUOY", "WATER", "WATER"])
        }),
        "north river checkpoint needs exact native31 Buoy/Water art"
    );
    for (x, open) in CELLS {
        ensure!(
            city.blocks[40 + usize::from(x)] == 49,
            "north river gate closed base differs at meta{x},1"
        );
        ensure!(
            collision
                .get(&format!("{open:02x}"))
                .is_some_and(|quadrants| {
                    quadrants.len() == 4 && quadrants.iter().all(|kind| kind == "WATER")
                }),
            "north river gate opening changed native Water collision"
        );
    }
    ensure!(
        city.events.coord_events.len() == 8,
        "north river checkpoint must preserve eight installed coordinate events"
    );
    // Keep the twelve-tile upper pool isolated even from a Surf dismount and
    // remount on a tiny bank shelf. Arrival-based town reachability alone does
    // not inspect an island that is accessible only from this incoming river.
    for y in 0..=1 {
        for x in [31, 38] {
            ensure!(
                kind(pack, city, [x, y])? == "WALL",
                "north river gate needs permanent banks at {x},{y}"
            );
        }
    }
    for y in [1, 4] {
        for x in 32..=37 {
            ensure!(
                kind(pack, city, [x, y])? == "WATER"
                    && !city
                        .events
                        .coord_events
                        .iter()
                        .any(|event| event.x == x && event.y == y)
                    && !city
                        .objects
                        .iter()
                        .any(|object| object.x == x && object.y == y),
                "north river checkpoint approach{x},{y} is blocked or already scripted"
            );
        }
    }
    for label in [CHECK, REFRESH, OPEN, DENIED, DONE, TEXT] {
        ensure!(
            !city.scripts.contains_key(label)
                && !city.script_text_bodies.contains_key(label)
                && !pack.data.map_scripts.contains_key(label)
                && !pack.data.maps.values().any(|map| {
                    map.scripts.contains_key(label) || map.script_text_bodies.contains_key(label)
                }),
            "occupied north river checkpoint label {label}"
        );
    }
    Ok(())
}
