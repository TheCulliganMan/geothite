//! A painted woodland dining hall and guest loft north of Grand Marais.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, LoadedCompiledGamePack};
use crate::{authored_route_surface, northstar_lair_authoring as events, northstar_story_revision};

const CEDAR: &str = "MnMaraisCedarApproach";
const HALL: &str = "MnMothlightLodge";
const LOFT: &str = "MnMothlightLoft";
const FOUND: &str = "EVENT_MN_MOTHLIGHT_CAKE_FOUND";
const PAID: &str = "EVENT_MN_MOTHLIGHT_CAKE_REWARD";
const TILESET: &str = "mn_mothlight";

#[derive(Deserialize)]
struct Design {
    exterior_rows: Vec<Edit>,
    hall_rows: Vec<String>,
    loft_rows: Vec<String>,
}
#[derive(Deserialize)]
struct Edit {
    at: [usize; 2],
    blocks: String,
}

/// Adds Mothlight Lodge to the verified Crossings parent without changing saves.
///
/// # Errors
/// Returns an error for a different parent, occupied identities, inaccessible
/// entrances, invalid source artwork or an invalid resulting content pack.
pub fn build_mothlight_lodge(base: &LoadedCompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.pack_content_hash()?
            == "5ad096a303e84c34353f4ac61cca7e52184001f8df71c9a9de9f0cabfa115aff",
        "Mothlight requires the selected Crossings parent"
    );
    let mut pack = base.pack().clone();
    let design: Design = serde_json::from_str(include_str!(
        "../../../planning/minnesota/landmarks/mothlight-lodge/layout.json"
    ))?;
    crate::mothlight_art::install(&mut pack)?;
    for flag in [FOUND, PAID] {
        events::flag(&mut pack, flag)?;
    }
    let mut cedar = pack.data.maps.get(CEDAR).context("Cedar approach")?.clone();
    ensure!(
        cedar.attributes.width == 20
            && cedar.attributes.height == 12
            && cedar.events.warps.len() == 11,
        "unexpected Cedar geography"
    );
    for edit in &design.exterior_rows {
        for (dx, value) in edit.blocks.split_whitespace().enumerate() {
            let [x, y] = edit.at;
            let index = y * 20 + x + dx;
            ensure!(
                x + dx < 20 && y < 12 && cedar.blocks[index] == 5,
                "lodge must occupy only the reserved pines"
            );
            cedar.blocks[index] = u16::from_str_radix(value, 16)?;
        }
    }
    cedar.attributes.tileset_name = "mn_mothlight_exterior".into();
    authoring::warp(&mut cedar, (30, 9), "MN_MOTHLIGHT_LODGE", 1);
    authoring::text(
        &mut cedar,
        "MnMothlightTrailSignText",
        &["MOTHLIGHT LODGE", "Hot supper. Dry socks."],
    );
    cedar.scripts.insert(
        "MnMothlightTrailSign".into(),
        json!([c("jumptext", &["MnMothlightTrailSignText"])]),
    );
    events::read(&mut cedar, [34, 10], "MnMothlightTrailSign");
    let occupied = cedar
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let reach = authored_route_surface::walking_map_distances(&pack, &cedar, [26, 23], &occupied)?;
    ensure!(
        reach.contains_key(&[30, 10]),
        "lodge porch must be reachable from Grand Marais"
    );
    northstar_story_revision::publish(&mut pack, cedar)?;

    let template = pack
        .data
        .maps
        .values()
        .find(|m| m.attributes.tileset_name == "traditional_house")
        .context("traditional house template")?
        .id
        .clone();
    let mut hall = authoring::blank(&pack, &template, HALL, "MN_MOTHLIGHT_LODGE")?;
    authoring::blocks(&mut hall, 13, 15, &design.hall_rows.join("\n"))?;
    hall.attributes.tileset_name = TILESET.into();
    hall.attributes.border_block = 0;
    hall.attributes.environment = Some("INDOOR".into());
    hall.attributes.music = Some("MUSIC_ECRUTEAK_CITY".into());
    hall.attributes.palette = Some("PALETTE_DAY".into());
    hall.attributes.time_of_day = Some("day".into());
    authoring::warp(&mut hall, (13, 29), "MN_MARAIS_CEDAR_APPROACH", 12);
    authoring::warp(&mut hall, (20, 8), "MN_MOTHLIGHT_LOFT", 1);
    let text: BTreeMap<String, Vec<String>> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/landmarks/mothlight-lodge/dialogue.json"
    ))?;
    for (label, lines) in &text {
        authoring::text(
            &mut hall,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    for (script, sprite, at, label) in [
        (
            "MnMothlightHost",
            "SPRITE_TEACHER",
            [8, 16],
            "MnMothlightHostText",
        ),
        (
            "MnMothlightRest",
            "SPRITE_GRANNY",
            [10, 9],
            "MnMothlightRestText",
        ),
        (
            "MnMothlightGuest",
            "SPRITE_GENTLEMAN",
            [17, 16],
            "MnMothlightGuestText",
        ),
        (
            "MnMothlightCook",
            "SPRITE_FISHER",
            [2, 17],
            "MnMothlightCookText",
        ),
    ] {
        authoring::npc(&pack, &mut hall, script, sprite, at[0], at[1])?;
        hall.scripts
            .insert(script.into(), json!([c("jumptextfaceplayer", &[label])]));
    }
    hall.scripts.insert(
        "MnMothlightHost".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[PAID]),
            c("iftrue", &["MnMothlightAfter"]),
            c("checkevent", &[FOUND]),
            c("iftrue", &["MnMothlightReward"]),
            c("jumptext", &["MnMothlightHostText"])
        ]),
    );
    hall.scripts.insert(
        "MnMothlightReward".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnMothlightSolvedText"]),
            c("promptbutton", &[]),
            c("verbosegiveitem", &["RARE_CANDY", "1"]),
            c("iffalse", &["MnMothlightFull"]),
            c("setevent", &[PAID]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for (script, label) in [
        ("MnMothlightAfter", "MnMothlightAfterText"),
        ("MnMothlightFull", "MnMothlightFullText"),
    ] {
        hall.scripts
            .insert(script.into(), json!([c("jumptext", &[label])]));
    }
    hall.scripts.insert(
        "MnMothlightRest".into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnMothlightRestText"]),
            c("promptbutton", &[]),
            c("special", &["HealParty"]),
            c("writetext", &["MnMothlightRestedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    hall.scripts.insert(
        "MnMothlightFindCake".into(),
        json!([
            c("setevent", &[FOUND]),
            c("jumptext", &["MnMothlightFoundText"])
        ]),
    );
    for (at, script, label) in [
        ([3, 13], "MnMothlightCrumbs", "MnMothlightCrumbsText"),
        ([2, 25], "MnMothlightBook", "MnMothlightBookText"),
        ([12, 7], "MnMothlightHearth", "MnMothlightHearthText"),
        ([6, 7], "MnMothlightMural", "MnMothlightMuralText"),
    ] {
        hall.scripts
            .insert(script.into(), json!([c("jumptext", &[label])]));
        events::read(&mut hall, at, script);
    }
    events::read(&mut hall, [5, 25], "MnMothlightFindCake");
    events::publish(&mut pack, hall, CEDAR, [13, 28])?;
    let loft_template = pack
        .data
        .maps
        .values()
        .find(|m| m.attributes.tileset_name == "house")
        .context("house template")?
        .id
        .clone();
    let mut loft = authoring::blank(&pack, &loft_template, LOFT, "MN_MOTHLIGHT_LOFT")?;
    ensure!(
        loft.attributes.tileset_name == "house",
        "native loft furniture"
    );
    authoring::blocks(&mut loft, 8, 6, &design.loft_rows.join("\n"))?;
    loft.attributes.tileset_name = "mn_mothlight_loft".into();
    loft.attributes.border_block = 0;
    loft.attributes.music = Some("MUSIC_ECRUTEAK_CITY".into());
    authoring::warp(&mut loft, (12, 8), "MN_MOTHLIGHT_LODGE", 2);
    authoring::speech(
        &pack,
        &mut loft,
        "Sleeper",
        "SPRITE_POKEFAN_M",
        (5, 6),
        &[
            "I'm not snoring. I'm keeping the moths away.",
            "...Is it working?",
        ],
    )?;
    events::publish(&mut pack, loft, CEDAR, [13, 8])?;
    for (name, origin) in [(HALL, [13, 28]), (LOFT, [13, 8])] {
        let room = &pack.data.maps[name];
        let occupied = room
            .objects
            .iter()
            .map(|o| [o.x, o.y])
            .collect::<BTreeSet<_>>();
        let paths = authored_route_surface::walking_map_distances(&pack, room, origin, &occupied)?;
        for object in &room.objects {
            ensure!(
                [
                    [object.x - 1, object.y],
                    [object.x + 1, object.y],
                    [object.x, object.y - 1],
                    [object.x, object.y + 1]
                ]
                .iter()
                .any(|p| paths.contains_key(p)),
                "unreachable lodge guest {}",
                object.script
            );
            ensure!(
                crate::authored_town_geometry::kind(&pack, room, [object.x, object.y])? == "FLOOR",
                "guest stands on furniture"
            );
        }
        for event in &room.events.bg_events {
            ensure!(
                [
                    [event.x - 1, event.y],
                    [event.x + 1, event.y],
                    [event.x, event.y - 1],
                    [event.x, event.y + 1]
                ]
                .iter()
                .any(|p| paths.contains_key(p)),
                "unreachable lodge discovery {}",
                event.script
            );
        }
    }
    ensure!(
        pack.audio_manifest == base.pack().audio_manifest,
        "lodge must retain all audio"
    );
    for (name, map) in &base.pack().data.maps {
        ensure!(
            name == CEDAR || pack.data.maps.get(name) == Some(map),
            "unrelated map changed: {name}"
        );
    }
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
