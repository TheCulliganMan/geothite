//! A neighbor's levee notebook and one retryable, native care-supply gift.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::MapModule;
use crate::grand_marais_modpack::authoring;

pub(crate) const READ: &str = "EVENT_MN_MOORHEAD_GAUGE_READ";
pub(crate) const GIVEN: &str = "EVENT_MN_MOORHEAD_DRY_KIT_GIVEN";
const HOST: &str = "MN_MOORHEAD_CLINIC_VOLUNTEER";
const BOARD: &str = "MN_MOORHEAD_RIVER_BOARD";

pub(crate) fn validate(source: &Value) -> Result<()> {
    ensure!(
        source["id"] == "MOORHEAD_DRY_KIT"
            && source["room"] == "MnMoorheadPokecenter1F"
            && source["actor"] == HOST
            && source["gauge_script"] == BOARD
            && source["read_flag"] == READ
            && source["once_flag"] == GIVEN
            && source["reward_item"] == "FULL_HEAL"
            && source["reward_quantity"] == 1,
        "Moorhead notebook must retain the real marker and one retryable FULL HEAL"
    );
    Ok(())
}

pub(crate) fn marker(map: &mut MapModule) -> Result<()> {
    ensure!(
        map.id == "MnMoorhead"
            && map
                .events
                .bg_events
                .iter()
                .filter(|e| e.script == BOARD && e.x == 12 && e.y == 4)
                .count()
                == 1
            && map.scripts.contains_key(BOARD),
        "Moorhead marker must be a real, reachable native placard"
    );
    let c = authoring::command;
    map.scripts.insert(
        BOARD.into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MN_MOORHEAD_RIVER_BOARDText"]),
            c("waitbutton", &[]),
            c("setevent", &[READ]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    Ok(())
}

pub(crate) fn volunteer(map: &mut MapModule, source: &Value) -> Result<()> {
    validate(source)?;
    ensure!(
        map.id == "MnMoorheadPokecenter1F"
            && map.objects.iter().filter(|o| o.script == HOST).count() == 1,
        "one actual clinic volunteer must keep the notebook"
    );
    let c = authoring::command;
    map.scripts.insert(
        HOST.into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[GIVEN]),
            c("iftrue", &["MnMoorheadKitRepeat"]),
            c("checkevent", &[READ]),
            c("iffalse", &["MnMoorheadKitFirst"]),
            c("writetext", &["MnMoorheadKitNotebookText"]),
            c("waitbutton", &[]),
            c("giveitem", &["FULL_HEAL", "1"]),
            c("iffalse", &["MnMoorheadKitFull"]),
            c("setevent", &[GIVEN]),
            c("writetext", &["MnMoorheadKitRewardText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnMoorheadKitFirst".into(),
        json!([
            c("writetext", &["MN_MOORHEAD_CLINIC_VOLUNTEERText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for (suffix, field) in [
        ("Notebook", "notebook_lines"),
        ("Reward", "reward_lines"),
        ("Full", "full_lines"),
        ("Repeat", "repeat_lines"),
    ] {
        let rows: Vec<String> = serde_json::from_value(source[field].clone())
            .with_context(|| format!("Moorhead notebook {field}"))?;
        ensure!(
            !rows.is_empty()
                && rows
                    .iter()
                    .all(|r| !r.is_empty() && r.chars().count() <= 18),
            "Moorhead notebook needs readable native text rows"
        );
        let label = format!("MnMoorheadKit{suffix}Text");
        authoring::text(
            map,
            &label,
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        if matches!(suffix, "Full" | "Repeat") {
            map.scripts.insert(
                format!("MnMoorheadKit{suffix}"),
                json!([
                    c("writetext", &[&label]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("end", &[])
                ]),
            );
        }
    }
    Ok(())
}
