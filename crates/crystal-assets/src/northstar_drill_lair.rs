//! A witnessed working drill, a pump maze and the crew's separate recovery quest.
use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

const MINE: &str = "MnR09Cave1F";
const APPROACH: &str = "MN_IRON_DRILL_APPROACH";
const GALLERY: &str = "MN_IRON_PUMP_GALLERY";
const CORE: &str = "MN_IRON_DRILL_CORE";
pub(super) const SEEN: &str = "EVENT_MN_DRILL_TEST_WITNESSED";
pub(super) const FEED: &str = "EVENT_MN_DRILL_FEED_ISOLATED";
pub(super) const WON: &str = "EVENT_MN_REED_MINE_DEFEATED";
pub(super) const OFF: &str = "EVENT_MN_DRILL_STOPPED";
const FOREMAN: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
pub(super) const RECOVERED: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
pub(super) const REED: &str = "MN_NDRILL_REED";
pub(super) const CREW: &str = "MN_NDRILL_FINN";

/// Extends the public Iron Range mine with a crosscut, pump maze and drill room.
///
/// The original foreman battle opens the lower workings. Isolating the feed
/// opens two physical shutters. The player witnesses native piston frames,
/// sound and shaking, defeats Reed and separately stops the drive. A rescued
/// crew member offers the meter and gives the next destination in person.
/// Reading the old mine log or interacting with its empty case cannot grant it.
/// All existing art and audio bytes are retained.
///
/// # Errors
/// Returns an error for incompatible prior story content, occupied identities,
/// invalid layouts, text or native scripts, or failed runtime verification.
pub fn build_northstar_drill_lair(
    base: &CompiledGamePack,
    events: &str,
    approach: &str,
    gallery: &str,
    core: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|m| m == "minnesota-northstar-ice-lair-v2"),
        "install the acted ICE chapter first"
    );
    ensure!(
        (!base
            .report
            .manifests
            .iter()
            .any(|m| m == "northstar-minnesota-only-v1")
            || crate::northstar_current_story::is_composing(base)),
        "bind the final regional cast after all story chapters"
    );
    let source: Value = serde_json::from_str(events)?;
    ensure!(
        source["schema_version"] == 1 && source["id"] == "northstar-drill-lair-v2",
        "supported drilling source required"
    );
    let mut pack = base.clone();
    for flag in [SEEN, FEED, WON, OFF] {
        a::flag(&mut pack, flag)?;
    }
    a::trainer(
        &mut pack,
        "MN_REED_MINE",
        "RIVAL1",
        "REED",
        &[("CROOKVEIL", 34), ("GOOSNIDE", 35), ("PELLETOR", 35)],
    )?;
    let mut mine = pack.data.maps.get(MINE).context("public mine")?.clone();
    install_public(&pack, &mut mine, &source)?;
    let arrival = i16::try_from(mine.events.warps.len() + 1)?;
    let mine_constant = mine
        .attributes
        .map_constant
        .clone()
        .context("public mine constant")?;
    authoring::warp(&mut mine, (19, 9), APPROACH, 2);
    authoring::warp(&mut mine, (19, 11), APPROACH, 2);
    let mut approach = a::room(&pack, &serde_json::from_str(approach)?, APPROACH)?;
    ensure!(approach.id == "MnIronDrillApproach", "crosscut identity");
    authoring::warp(&mut approach, (7, 21), &mine_constant, arrival);
    authoring::warp(&mut approach, (6, 21), &mine_constant, arrival);
    authoring::warp(&mut approach, (25, 5), GALLERY, 2);
    authoring::warp(&mut approach, (24, 5), GALLERY, 2);
    let mut gallery = a::room(&pack, &serde_json::from_str(gallery)?, GALLERY)?;
    ensure!(gallery.id == "MnIronPumpGallery", "gallery identity");
    authoring::warp(&mut gallery, (5, 18), APPROACH, 4);
    authoring::warp(&mut gallery, (4, 19), APPROACH, 4);
    authoring::warp(&mut gallery, (25, 4), CORE, 2);
    authoring::warp(&mut gallery, (24, 5), CORE, 2);
    install_gallery(&mut gallery, &source)?;
    let mut core = a::room(&pack, &serde_json::from_str(core)?, CORE)?;
    ensure!(core.id == "MnIronDrillCore", "drill-room identity");
    authoring::warp(&mut core, (5, 14), GALLERY, 4);
    authoring::warp(&mut core, (4, 15), GALLERY, 4);
    crate::northstar_drill_scene::install(&pack, &mut core, &source)?;
    a::publish(&mut pack, approach, MINE, [6, 21])?;
    a::publish(&mut pack, gallery, MINE, [4, 19])?;
    a::publish(&mut pack, core, MINE, [4, 15])?;
    crate::ely_league_rooms::finish(&pack, &mut mine)?;
    crate::authored_town_publication::replace(&mut pack, mine)?;
    crate::northstar_story_bridges::install(&mut pack, &source)?;
    pack.report.maps = pack.data.maps.len();
    pack.report
        .manifests
        .push("minnesota-northstar-drill-lair-v2".into());
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

pub(super) fn put(map: &mut MapModule, label: &str, commands: Vec<Value>) -> Result<()> {
    ensure!(
        map.scripts.insert(label.into(), json!(commands)).is_none(),
        "occupied drilling program {label}"
    );
    Ok(())
}

pub(super) fn jump(map: &mut MapModule, script: &str, text: &str) -> Result<()> {
    put(map, script, vec![c("jumptext", &[text])])
}

fn install_public(pack: &CompiledGamePack, map: &mut MapModule, source: &Value) -> Result<()> {
    ensure!(
        map.attributes.width == 12 && map.attributes.height == 11 && map.blocks[5 * 12 + 9] == 2,
        "reviewed public-mine stair preimage"
    );
    a::text(
        map,
        source,
        &[
            "MnNDrillScoutQuestText",
            "MnNDrillMeterDeniedText",
            "MnNDrillEmptyCaseText",
            "MnNDrillPublicAfterText",
        ],
    )?;
    // A sealed lower shaft opens after the existing, prerequisite-gated battle.
    // Its passive return is adjacent ordinary floor before every entry callback.
    map.blocks[5 * 12 + 9] = 0x2d;
    put(
        map,
        "MnNDrillPublicTiles",
        vec![
            c("checkevent", &[FOREMAN]),
            c("iffalse", &["MnNDrillPublicEnd"]),
            c("changeblock", &["18", "10", "7"]),
            c("endcallback", &[]),
        ],
    )?;
    a::callback(map, "MAPCALLBACK_TILES", "MnNDrillPublicTiles");
    // Open it immediately when the player wins, as well as on later visits.
    let after = map
        .scripts
        .get_mut("MnIronRangeForemanAfter")
        .and_then(Value::as_array_mut)
        .context("original foreman continuation")?;
    after.insert(0, c("changeblock", &["18", "10", "7"]));
    put(map, "MnNDrillPublicEnd", vec![c("end", &[])])?;
    let scout = map
        .objects
        .iter_mut()
        .find(|o| o.x == 19 && o.y == 7)
        .context("native mine scout")?;
    scout.sprite = "SPRITE_POKEFAN_F".into();
    scout.object_identifier = Some("MN_NDRILL_NELL".into());
    scout.script = "MN_NDRILL_NELL".into();
    put(
        map,
        "MN_NDRILL_NELL",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[OFF]),
            c("iftrue", &["MnNDrillPublicAfter"]),
            c("jumptext", &["MnNDrillScoutQuestText"]),
        ],
    )?;
    jump(map, "MnNDrillPublicAfter", "MnNDrillPublicAfterText")?;
    // The old found case is no longer the source of the next quest.
    ensure!(
        map.scripts.contains_key("MN_R09_SURVEY_CACHE"),
        "old case program"
    );
    map.scripts.insert(
        "MN_R09_SURVEY_CACHE".into(),
        json!([
            c("checkevent", &[OFF]),
            c("iffalse", &["MnNDrillMeterDenied"]),
            c("jumptext", &["MnNDrillEmptyCaseText"]),
        ]),
    );
    jump(map, "MnNDrillMeterDenied", "MnNDrillMeterDeniedText")?;
    // The archived old success branch remains unreachable; no old entry point
    // may award RECOVERED outside the rescued crew member's new program.
    for program in map.scripts.values_mut().filter_map(Value::as_array_mut) {
        program.retain(|row| *row != c("setevent", &[RECOVERED]));
    }
    crate::ely_league_rooms::finish(pack, map)
}

fn install_gallery(map: &mut MapModule, source: &Value) -> Result<()> {
    a::text(
        map,
        source,
        &[
            "MnNDrillGalleryText",
            "MnNDrillGalleryDoneText",
            "MnNDrillGalleryAfterText",
        ],
    )?;
    let opened = vec![
        c("changeblock", &["24", "6", "13"]),
        c("changeblock", &["14", "16", "13"]),
    ];
    let mut callback = vec![
        c("checkevent", &[FEED]),
        c("iffalse", &["MnNDrillGalleryEnd"]),
    ];
    callback.extend(opened.clone());
    callback.push(c("endcallback", &[]));
    put(map, "MnNDrillGalleryTiles", callback)?;
    a::callback(map, "MAPCALLBACK_TILES", "MnNDrillGalleryTiles");
    let mut feed = vec![
        c("checkevent", &[FEED]),
        c("iftrue", &["MnNDrillGalleryAfter"]),
        c("opentext", &[]),
        c("writetext", &["MnNDrillGalleryText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnNDrillGalleryDeclined"]),
        c("closetext", &[]),
    ];
    feed.extend(opened);
    feed.push(c("refreshmap", &[]));
    feed.extend(a::say("MnNDrillGalleryDoneText"));
    feed.extend([c("setevent", &[FEED]), c("end", &[])]);
    put(map, "MnNDrillGalleryFeed", feed)?;
    put(
        map,
        "MnNDrillGalleryDeclined",
        vec![c("closetext", &[]), c("end", &[])],
    )?;
    put(map, "MnNDrillGalleryEnd", vec![c("end", &[])])?;
    jump(map, "MnNDrillGalleryAfter", "MnNDrillGalleryAfterText")?;
    a::read(map, [25, 17], "MnNDrillGalleryFeed");
    Ok(())
}

pub(super) fn entrances(map: &mut MapModule) -> Result<()> {
    // A complete cross-section catches either side of the safe stair landing.
    // Every path into the working half crosses it; all scripts walk the player
    // on the same clear row before taking the native witnessed camera pose.
    for x in 3u16..=20 {
        let label = format!("MnNDrillEnter{x}");
        let movement = format!("MnNDrillWatchFrom{x}");
        let mut steps = Vec::new();
        let direction = if x < 9 { "RIGHT" } else { "LEFT" };
        steps.extend((0..x.abs_diff(9)).map(|_| c("step", &[direction])));
        steps.extend((0..3).map(|_| c("step", &["UP"])));
        steps.push(c("step_end", &[]));
        put(map, &movement, steps)?;
        put(
            map,
            &label,
            vec![
                c("checkevent", &[SEEN]),
                c("iftrue", &["MnNDrillEnd"]),
                c("applymovement", &["PLAYER", &movement]),
                c("sjump", &["MnNDrillWitness"]),
            ],
        )?;
        map.events.coord_events.push(CoordEvent {
            x,
            y: 12,
            scene_id: "-1".into(),
            script_name: label,
        });
    }
    Ok(())
}
