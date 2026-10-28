//! Native SQ03 check-in, guarded winter departure and remembered far-camp arrival.
use anyhow::{Context, Result, ensure};
use crystal_core::map::CoordEvent;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::{Value, json};

use crate::authored_town_design::Town;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule, MapScriptSectionCommand};

pub(crate) const REPORT: &str = "EVENT_MN_BORDER_REPORT";
pub(crate) const ARRIVED: &str = "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL";
pub(crate) const GIVEN: &str = "EVENT_MN_BORDER_CIRCUIT_REWARDED";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const HOST: &str = "MN_FALLS_RANGER";

pub(crate) fn validate(q: &Value) -> Result<()> {
    let mut required = (1..=8).map(|i| format!("BADGE_{i}")).collect::<Vec<_>>();
    required.extend([REPORT.into(), SIGNAL.into()]);
    ensure!(
        q["id"] == "SQ03"
            && q["ranger"] == HOST
            && q["report"] == REPORT
            && q["arrived"] == ARRIVED
            && q["rewarded"] == GIVEN
            && q["reward_item"] == "PP_UP"
            && q["reward_quantity"] == 1
            && q["check_in_badge"] == "ENGINE_GLACIERBADGE"
            && q["outlet_requires"] == json!(required)
            && q["arrival_map"] == "MnBoundaryWaters"
            && q["arrival_mode"] == "MAPCALLBACK_NEWMAP"
            && q["arrival_tiles"] == json!([[0, 24], [0, 25]])
            && q["arrival_facing"] == "RIGHT",
        "retain SQ03 authority, guards and real eastbound far-camp arrival"
    );
    Ok(())
}

fn text(map: &mut MapModule, q: &Value, key: &str) -> Result<String> {
    let rows = q[key]
        .as_array()
        .context("Falls dialogue rows")?
        .iter()
        .map(|v| v.as_str().context("Falls dialogue string"))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        !rows.is_empty()
            && rows
                .iter()
                .all(|s| !s.is_empty() && s.chars().count() <= 18),
        "Falls dialogue must fit native rows"
    );
    let label = format!("MnFalls{key}Text");
    authoring::text(map, &label, &rows);
    Ok(label)
}

fn checks(failure: &str) -> Vec<Value> {
    let c = authoring::command;
    let mut commands = Vec::new();
    for badge in &BADGE_ENGINE_FLAGS[..8] {
        commands.extend([c("checkflag", &[badge]), c("iffalse", &[failure])]);
    }
    for flag in [REPORT, SIGNAL] {
        commands.extend([c("checkevent", &[flag]), c("iffalse", &[failure])]);
    }
    commands
}

pub(crate) fn gate(map: &mut MapModule, q: &Value) -> Result<()> {
    let c = authoring::command;
    let label = text(map, q, "denied_lines")?;
    let mut commands = vec![
        c("readvar", &["VAR_FACING"]),
        c("ifnotequal", &["RIGHT", "MnFallsTrailEnd"]),
    ];
    commands.extend(checks("MnFallsTrailDenied"));
    commands.push(c("end", &[]));
    map.scripts
        .insert("MnFallsTrailCheck".into(), json!(commands));
    map.scripts
        .insert("MnFallsTrailEnd".into(), json!([c("end", &[])]));
    map.scripts.insert(
        "MnFallsTrailDenied".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &[&label]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("applymovement", &["PLAYER", "MnFallsTrailBack"]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnFallsTrailBack".into(),
        json!([c("step", &["LEFT"]), c("step_end", &[])]),
    );
    for y in [14, 15] {
        map.events.coord_events.push(CoordEvent {
            x: 29,
            y,
            scene_id: String::new(),
            script_name: "MnFallsTrailCheck".into(),
        });
    }
    Ok(())
}

pub(crate) fn ranger(map: &mut MapModule, q: &Value) -> Result<()> {
    ensure!(
        map.objects.iter().any(|o| o.script == HOST),
        "Falls ranger must be an actual clinic resident"
    );
    let c = authoring::command;
    let complete = text(map, q, "complete_lines")?;
    let reward = text(map, q, "reward_lines")?;
    let registered = text(map, q, "registered_lines")?;
    let register = text(map, q, "register_lines")?;
    map.scripts.insert(
        HOST.into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[GIVEN]),
            c("iftrue", &["MnFallsRepeat"]),
            c("checkflag", &["ENGINE_GLACIERBADGE"]),
            c("iffalse", &["MnFallsBadge"]),
            c("checkevent", &[REPORT]),
            c("iffalse", &["MnFallsRegister"]),
            c("checkevent", &[ARRIVED]),
            c("iffalse", &["MnFallsWaiting"]),
            c("writetext", &[&complete]),
            c("waitbutton", &[]),
            c("giveitem", &["PP_UP", "1"]),
            c("iffalse", &["MnFallsFull"]),
            c("setevent", &[GIVEN]),
            c("writetext", &[&reward]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnFallsRegister".into(),
        json!([
            c("writetext", &[&register]),
            c("yesorno", &[]),
            c("iffalse", &["MnFallsDecline"]),
            c("setevent", &[REPORT]),
            c("writetext", &[&registered]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for (suffix, key) in [
        ("Repeat", "repeat_lines"),
        ("Badge", "badge_lines"),
        ("Waiting", "waiting_lines"),
        ("Full", "full_lines"),
        ("Decline", "decline_lines"),
    ] {
        let label = text(map, q, key)?;
        map.scripts.insert(
            format!("MnFalls{suffix}"),
            json!([
                c("writetext", &[&label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    Ok(())
}

pub(crate) fn arrival(pack: &mut CompiledGamePack, town: &Town) -> Result<()> {
    let name = "MnBoundaryWaters";
    let mut new = authoring::blank(pack, name, name, "MN_BOUNDARY_WATERS")?;
    let original = pack.data.maps.get(name).context("existing far-camp hub")?;
    new.attributes = original.attributes.clone();
    new.blocks = original.blocks.clone();
    let c = authoring::command;
    new.scripts.insert(
        "MnFallsVoyageursArrival".into(),
        json!([
            c("readvar", &["VAR_XCOORD"]),
            c("ifnotequal", &["0", "MnFallsArrivalEnd"]),
            c("readvar", &["VAR_YCOORD"]),
            c("ifequal", &["24", "MnFallsArrivalQualify"]),
            c("ifnotequal", &["25", "MnFallsArrivalEnd"]),
            c("sjump", &["MnFallsArrivalQualify"])
        ]),
    );
    let mut commands = vec![
        c("readvar", &["VAR_FACING"]),
        c("ifnotequal", &["RIGHT", "MnFallsArrivalEnd"]),
    ];
    commands.extend(checks("MnFallsArrivalEnd"));
    commands.extend([c("setevent", &[ARRIVED]), c("endcallback", &[])]);
    new.scripts
        .insert("MnFallsArrivalQualify".into(), json!(commands));
    new.scripts
        .insert("MnFallsArrivalEnd".into(), json!([c("endcallback", &[])]));
    crate::authored_town_scripts::finish(pack, &mut new, town)?;
    let mut revised = original.clone();
    crate::authored_city_expansion::merge_new_content(&mut revised, new)?;
    revised
        .map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec![
                "MAPCALLBACK_NEWMAP".into(),
                "MnFallsVoyageursArrival".into(),
            ],
            command_index: revised.map_script_section_commands.len(),
        });
    crate::authored_town_publication::replace(pack, revised)
}
