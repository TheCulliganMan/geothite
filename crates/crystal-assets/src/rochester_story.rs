//! Acknowledged originals delivery, separate from the first gym battle.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::MapModule;
use crate::grand_marais_modpack::authoring;

pub(super) fn install(map: &mut MapModule, source: &Value) -> Result<()> {
    let milestone = source["milestones"]
        .as_array()
        .context("milestones")?
        .iter()
        .find(|m| m["id"] == "MS01")
        .context("Rochester delivery milestone")?;
    ensure!(
        milestone["location"] == "MnRochester"
            && milestone["requires"]
                == json!(["EVENT_MN_STARTER_RECEIVED", "EVENT_MN_R01_PARCEL_RECEIVED"])
            && milestone["sets"] == json!(["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
        "canonical originals-delivery prerequisite and completion required"
    );
    let pages: Vec<Vec<String>> = serde_json::from_value(milestone["dialogue_pages"].clone())?;
    ensure!(
        (2..=6).contains(&pages.len())
            && pages
                .iter()
                .all(|p| p.len() == 2 && p.iter().all(|s| s.is_ascii() && s.len() <= 18)),
        "two to six complete native delivery pages"
    );
    authoring::text(
        map,
        "MnRCDeliveryText",
        &pages
            .iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    authoring::text(
        map,
        "MnRCMissingText",
        &[
            "DR TAMARACK keeps",
            "the old readings.",
            "Bring the originals",
            "from WINONA.",
        ],
    );
    authoring::text(
        map,
        "MnRCDeliveryAfterText",
        &[
            "We kept your copy.",
            "VAREL is ready.",
            "The missing meters",
            "were meant for us.",
        ],
    );
    let c = authoring::command;
    map.scripts.insert(
        "MN_ROCHESTER_RECIPIENT".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
            c("iftrue", &["MnRCDeliveryAfter"]),
            c("checkevent", &["EVENT_MN_STARTER_RECEIVED"]),
            c("iffalse", &["MnRCMissing"]),
            c("checkevent", &["EVENT_MN_R01_PARCEL_RECEIVED"]),
            c("iffalse", &["MnRCMissing"]),
            c("opentext", &[]),
            c("writetext", &["MnRCDeliveryText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnRCMissing".into(),
        json!([c("jumptext", &["MnRCMissingText"])]),
    );
    map.scripts.insert(
        "MnRCDeliveryAfter".into(),
        json!([c("jumptext", &["MnRCDeliveryAfterText"])]),
    );
    Ok(())
}
