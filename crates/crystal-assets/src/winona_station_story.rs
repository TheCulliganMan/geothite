//! The depot's winter register, a mason's memory and a retryable dry kit.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::MapModule;
use crate::grand_marais_modpack::authoring;

pub(crate) const VISITED: &str = "EVENT_MN_STATION_WINONA_VISITED";
pub(crate) const READ: &str = "EVENT_MN_WINONA_DEPOT_LOG_READ";
pub(crate) const HEARD: &str = "EVENT_MN_WINONA_DEPOT_MASON_HEARD";
pub(crate) const GIVEN: &str = "EVENT_MN_WINONA_DEPOT_DRY_KIT_GIVEN";

pub(crate) fn install(map: &mut MapModule, source: &Value) -> Result<()> {
    ensure!(
        map.id == "MnWinonaStation"
            && source["id"] == "WINONA_WINTER_REGISTER"
            && source["visit_flag"] == VISITED
            && source["read_flag"] == READ
            && source["heard_flag"] == HEARD
            && source["gift_flag"] == GIVEN
            && source["reward_item"] == "ICE_HEAL"
            && source["reward_quantity"] == 1,
        "Winona depot must retain the two remembered encounters and one ICE HEAL"
    );
    let c = authoring::command;
    for (script, flag) in [
        ("MN_WINONA_STATION_ICE_LOG", READ),
        ("MN_WINONA_STATION_MASON", HEARD),
    ] {
        ensure!(
            map.scripts.contains_key(script),
            "missing depot encounter {script}"
        );
        let mut commands = Vec::new();
        if flag == HEARD {
            commands.push(c("faceplayer", &[]));
        }
        commands.extend([
            c("opentext", &[]),
            c("writetext", &[&format!("{script}Text")]),
            c("waitbutton", &[]),
            c("setevent", &[flag]),
            c("closetext", &[]),
            c("end", &[]),
        ]);
        map.scripts.insert(script.into(), json!(commands));
    }
    ensure!(
        map.objects
            .iter()
            .filter(|o| o.script == "MN_WINONA_STATION_PORTER")
            .count()
            == 1,
        "one real depot porter required"
    );
    map.scripts.insert(
        "MN_WINONA_STATION_PORTER".into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[GIVEN]),
            c("iftrue", &["MnWinonaDepotRepeat"]),
            c("checkevent", &[READ]),
            c("iffalse", &["MnWinonaDepotFirst"]),
            c("checkevent", &[HEARD]),
            c("iffalse", &["MnWinonaDepotFirst"]),
            c("giveitem", &["ICE_HEAL", "1"]),
            c("iffalse", &["MnWinonaDepotFull"]),
            c("setevent", &[GIVEN]),
            c("sjump", &["MnWinonaDepotReward"])
        ]),
    );
    map.scripts.insert(
        "MnWinonaDepotFirst".into(),
        json!([
            c("writetext", &["MN_WINONA_STATION_PORTERText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for (suffix, field) in [
        ("Reward", "reward_lines"),
        ("Full", "full_lines"),
        ("Repeat", "repeat_lines"),
    ] {
        let lines: Vec<String> = serde_json::from_value(source[field].clone())
            .with_context(|| format!("depot {field}"))?;
        ensure!(
            !lines.is_empty()
                && lines
                    .iter()
                    .all(|line| !line.is_empty() && line.chars().count() <= 18),
            "depot dialogue must fit the native text window"
        );
        let label = format!("MnWinonaDepot{suffix}Text");
        authoring::text(
            map,
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        map.scripts.insert(
            format!("MnWinonaDepot{suffix}"),
            json!([
                c("writetext", &[&label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    map.scripts.insert(
        "MnWinonaDepotFirstVisit".into(),
        json!([c("setevent", &[VISITED]), c("endcallback", &[])]),
    );
    Ok(())
}
