//! Player-carried rescue news and small, persistent shore recovery scenes.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{RESTORED, WORKER};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "northstar-rescue-and-shore-aftermath-v1";
const RADIO: &str = "EVENT_MN_SERA_RADIO_SENT";
const REPORTED: &str = "EVENT_MN_SERA_RADIO_REPORTED";
const NELL_HEARD: &str = "EVENT_MN_NELL_RADIO_ACKNOWLEDGED";
const DRILL_OFF: &str = "EVENT_MN_DRILL_STOPPED";
const WINONA_RELEASED: &str = "EVENT_MN_WINONA_SHORE_RELEASED";
const LEECH_RELEASED: &str = "EVENT_MN_LEECH_SHORE_RELEASED";

type Dialogue = BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>;

/// Adds an optional radio handoff and two restored-state shore interactions.
///
/// Rescue reports never complete the mine or restoration objectives. Existing
/// completed saves receive the appropriate later reply immediately. New actors
/// append to each map, keep every original position, and preserve walkable paths.
pub(crate) fn install(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|name| name == MANIFEST),
        "restoration aftermath already installed"
    );
    crate::northstar_shore_roster_repair::validate_partner(pack)?;
    let story: Value = serde_json::from_str(story_source)?;
    let dialogue: Dialogue =
        serde_json::from_value(story["refinement_aftermath"]["dialogue"].clone())?;
    for flag in [RADIO, REPORTED, NELL_HEARD, WINONA_RELEASED, LEECH_RELEASED] {
        a::flag(pack, flag)?;
    }
    sera(pack, &story, &dialogue)?;
    radio(pack, &dialogue)?;
    nell(pack, &dialogue)?;
    for shore in [
        Shore {
            map: "MnWinona",
            prefix: "MnRestoreWinona",
            worker: "MN_RESTORE_WINONA_WORKER",
            partner: "MN_RESTORE_WINONA_GILLPUP",
            worker_at: [24, 10],
            partner_at: [25, 11],
            entry: [16, 10],
            released: WINONA_RELEASED,
        },
        Shore {
            map: "MnLeechLake",
            prefix: "MnRestoreLeech",
            worker: "MN_RESTORE_LEECH_WORKER",
            partner: "MN_RESTORE_LEECH_GILLPUP",
            worker_at: [30, 27],
            partner_at: [31, 26],
            entry: [32, 38],
            released: LEECH_RELEASED,
        },
    ] {
        shoreline(pack, &dialogue, &shore)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn sera(pack: &mut CompiledGamePack, story: &Value, dialogue: &Dialogue) -> Result<()> {
    let mut map = map_with_texts(pack, dialogue, "MnStCloud")?;
    ensure!(
        map.scripts.get("MnArcSCAfter") == Some(&json!([c("jumptext", &["MnArcSCAfterText"])])),
        "changed Sera return program"
    );
    let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(
        story["dialogue_overhaul"]["texts"]["MnStCloud"]["MnArcSCReleasedText"].clone(),
    )?;
    ensure!(
        map.scripts.contains_key("MnArcSCReleasedText"),
        "existing Sera release text"
    );
    map.scripts.insert(
        "MnArcSCReleasedText".into(),
        authored_dialogue::compile(&pages)?,
    );
    put(
        &mut map,
        "MnArcSCAfter",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MnRefineSeraRestored"]),
            c("checkevent", &[DRILL_OFF]),
            c("iftrue", &["MnRefineSeraMineSafe"]),
            c("checkevent", &[RADIO]),
            c("iffalse", &["MnRefineSeraHandoff"]),
            c("checkevent", &[REPORTED]),
            c("iftrue", &["MnRefineSeraWaiting"]),
            c("opentext", &[]),
            c("writetext", &["MnRefineSeraHeardText"]),
            c("waitbutton", &[]),
            c("setevent", &[REPORTED]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    for kind in ["Restored", "MineSafe", "Handoff", "Waiting"] {
        let label = format!("MnRefineSera{kind}");
        put(
            &mut map,
            &label,
            vec![c("jumptext", &[&format!("{label}Text")])],
        );
    }
    crate::northstar_story_revision::publish(pack, map)
}

fn radio(pack: &mut CompiledGamePack, dialogue: &Dialogue) -> Result<()> {
    let mut map = map_with_texts(pack, dialogue, "MnStCloudStation")?;
    const CLERK: &str = "MN_ST_CLOUD_STATION_CLERK";
    let original = json!([c("jumptextfaceplayer", &["MN_ST_CLOUD_STATION_CLERKText"])]);
    ensure!(
        map.scripts.get(CLERK) == Some(&original),
        "changed station clerk"
    );
    map.scripts
        .insert("MnRefineRadioClerkOrdinary".into(), original);
    put(
        &mut map,
        CLERK,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MnRefineRadioClerkOrdinary"]),
            c("checkevent", &[WORKER]),
            c("iffalse", &["MnRefineRadioClerkOrdinary"]),
            c("checkevent", &[DRILL_OFF]),
            c("iftrue", &["MnRefineRadioSafe"]),
            c("checkevent", &[RADIO]),
            c("iftrue", &["MnRefineRadioRepeat"]),
            c("opentext", &[]),
            c("writetext", &["MnRefineRadioOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnRefineRadioDeclined"]),
            c("writetext", &["MnRefineRadioSentText"]),
            c("waitbutton", &[]),
            c("setevent", &[RADIO]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        &mut map,
        "MnRefineRadioDeclined",
        vec![
            c("writetext", &["MnRefineRadioDeclinedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    for kind in ["Repeat", "Safe"] {
        let label = format!("MnRefineRadio{kind}");
        put(
            &mut map,
            &label,
            vec![c("jumptext", &[&format!("{label}Text")])],
        );
    }
    crate::northstar_story_revision::publish(pack, map)
}

fn nell(pack: &mut CompiledGamePack, dialogue: &Dialogue) -> Result<()> {
    let mut map = map_with_texts(pack, dialogue, "MnR09Cave1F")?;
    const NELL: &str = "MN_NDRILL_NELL";
    let original = map
        .scripts
        .get(NELL)
        .context("Nell's existing mine program")?
        .clone();
    ensure!(
        original.as_array().is_some_and(|rows| rows.starts_with(&[
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MN_NDRILL_NELLArcReturn"])
        ])),
        "Nell's restoration return must remain first"
    );
    map.scripts.insert("MnRefineNellOriginal".into(), original);
    put(
        &mut map,
        NELL,
        vec![
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MnRefineNellOriginal"]),
            c("checkevent", &[DRILL_OFF]),
            c("iftrue", &["MnRefineNellOriginal"]),
            c("checkevent", &[RADIO]),
            c("iffalse", &["MnRefineNellOriginal"]),
            c("checkevent", &[NELL_HEARD]),
            c("iftrue", &["MnRefineNellOriginal"]),
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnRefineNellRadioText"]),
            c("waitbutton", &[]),
            c("setevent", &[NELL_HEARD]),
            c("closetext", &[]),
            c("sjump", &["MnRefineNellOriginal"]),
        ],
    );
    crate::northstar_story_revision::publish(pack, map)
}

struct Shore {
    map: &'static str,
    prefix: &'static str,
    worker: &'static str,
    partner: &'static str,
    worker_at: [u16; 2],
    partner_at: [u16; 2],
    entry: [u16; 2],
    released: &'static str,
}

fn shoreline(pack: &mut CompiledGamePack, dialogue: &Dialogue, shore: &Shore) -> Result<()> {
    let mut map = map_with_texts(pack, dialogue, shore.map)?;
    let original_objects = map.objects.clone();
    let before = reachable(pack, &map, shore.entry, false)?;
    for at in [shore.worker_at, shore.partner_at] {
        ensure!(
            kind(pack, &map, at)? == "FLOOR" && before.contains(&at),
            "reachable shore floor required"
        );
        ensure!(
            !map.events
                .warps
                .iter()
                .any(|event| [event.x, event.y] == at)
                && !map
                    .events
                    .bg_events
                    .iter()
                    .any(|event| [event.x, event.y] == at)
                && !map
                    .events
                    .coord_events
                    .iter()
                    .any(|event| [event.x, event.y] == at),
            "shore actor must not cover an existing event"
        );
    }
    a::actor(
        pack,
        &mut map,
        shore.worker,
        "SPRITE_POKEFAN_F",
        shore.worker_at,
    )?;
    a::actor(
        pack,
        &mut map,
        shore.partner,
        crate::northstar_shore_roster_repair::PARTNER_SPRITE,
        shore.partner_at,
    )?;
    let additions = BTreeSet::from([shore.worker_at, shore.partner_at]);
    let after = reachable(pack, &map, shore.entry, false)?;
    ensure!(
        before
            .difference(&additions)
            .all(|tile| after.contains(tile)),
        "shore actors must not sever any existing walkable path"
    );
    ensure!(
        map.objects.starts_with(&original_objects),
        "original shore actors must remain unchanged"
    );
    let prefix = shore.prefix;
    let actors = format!("{prefix}Actors");
    let end = format!("{prefix}ActorsEnd");
    put(
        &mut map,
        &actors,
        vec![
            c("disappear", &[shore.worker]),
            c("disappear", &[shore.partner]),
            c("checkevent", &[RESTORED]),
            c("iffalse", &[&end]),
            c("appear", &[shore.worker]),
            c("checkevent", &[shore.released]),
            c("iffalse", &[&end]),
            c("appear", &[shore.partner]),
            c("endcallback", &[]),
        ],
    );
    put(&mut map, &end, vec![c("endcallback", &[])]);
    a::callback(&mut map, "MAPCALLBACK_OBJECTS", &actors);
    // Continue restores the old save's actor roster and runs SPRITES, not
    // OBJECTS. Reconcile these appended actors there too, using the same
    // idempotent story flags instead of changing generic save restoration.
    a::callback(&mut map, "MAPCALLBACK_SPRITES", &actors);
    put(
        &mut map,
        shore.worker,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[RESTORED]),
            c("iffalse", &[&format!("{prefix}Unavailable")]),
            c("checkevent", &[shore.released]),
            c("iftrue", &[&format!("{prefix}After")]),
            c("opentext", &[]),
            c("writetext", &[&format!("{prefix}OfferText")]),
            c("yesorno", &[]),
            c("iffalse", &[&format!("{prefix}Declined")]),
            c("closetext", &[]),
            c("setevent", &[shore.released]),
            c("appear", &[shore.partner]),
            c("faceobject", &[shore.partner, "PLAYER"]),
            c("jumptext", &[&format!("{prefix}DoneText")]),
        ],
    );
    put(
        &mut map,
        &format!("{prefix}Unavailable"),
        vec![c("end", &[])],
    );
    put(
        &mut map,
        &format!("{prefix}After"),
        vec![c("jumptext", &[&format!("{prefix}AfterText")])],
    );
    put(
        &mut map,
        &format!("{prefix}Declined"),
        vec![
            c("writetext", &[&format!("{prefix}DeclinedText")]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        &mut map,
        shore.partner,
        vec![c("jumptextfaceplayer", &[&format!("{prefix}GillpupText")])],
    );
    if shore.map == "MnWinona" {
        const WATCHER: &str = "MN_W_RIVER_WATCHER";
        let original = map
            .scripts
            .get(WATCHER)
            .context("Winona river watcher")?
            .clone();
        ensure!(
            original == json!([c("jumptextfaceplayer", &["MN_W_RIVER_WATCHERText"])]),
            "changed Winona river watcher"
        );
        map.scripts
            .insert("MnRestoreWinonaWatcherBefore".into(), original);
        put(
            &mut map,
            WATCHER,
            vec![
                c("checkevent", &[RESTORED]),
                c("iffalse", &["MnRestoreWinonaWatcherBefore"]),
                c("jumptextfaceplayer", &["MnRestoreWinonaWatcherText"]),
            ],
        );
    }
    crate::northstar_story_revision::publish(pack, map)
}

fn map_with_texts(pack: &CompiledGamePack, dialogue: &Dialogue, name: &str) -> Result<MapModule> {
    let mut map = pack
        .data
        .maps
        .get(name)
        .context("existing aftermath map")?
        .clone();
    for (label, pages) in dialogue.get(name).context("canonical aftermath pages")? {
        ensure!(
            !map.scripts.contains_key(label),
            "occupied aftermath text {label}"
        );
        map.scripts
            .insert(label.clone(), authored_dialogue::compile(pages)?);
    }
    Ok(map)
}

fn put(map: &mut MapModule, label: &str, rows: Vec<Value>) {
    map.scripts.insert(label.into(), Value::Array(rows));
}
