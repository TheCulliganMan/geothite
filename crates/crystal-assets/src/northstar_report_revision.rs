//! Give the officer time to inspect the papers and react to the copied record.
use anyhow::{Context, Result, ensure};

use crate::MapModule;
use crate::grand_marais_modpack::authoring::command as c;

pub(crate) fn install(map: &mut MapModule) -> Result<()> {
    ensure!(map.id == "MnBemidji", "Bemidji report scene required");
    let body = map
        .scripts
        .get_mut("MN_BEMIDJI_OFFICER")
        .and_then(serde_json::Value::as_array_mut)
        .context("officer's record comparison")?;
    let open = body
        .iter()
        .position(|row| row == &c("opentext", &[]))
        .context("comparison textbox")?;
    ensure!(
        body.get(open + 1) == Some(&c("writetext", &["MnBemidjiComparisonText"])),
        "comparison text must follow the prerequisite checks"
    );
    let report = body
        .iter()
        .position(|row| row == &c("setevent", &["EVENT_MN_HEADWATERS_REPORT"]))
        .context("acknowledged report")?;
    let inspect = [
        c("showemote", &["EMOTE_QUESTION", "MN_BEMIDJI_OFFICER", "15"]),
        c("pause", &["15"]),
    ];
    let react = [
        c("showemote", &["EMOTE_SHOCK", "MN_BEMIDJI_OFFICER", "15"]),
        c("pause", &["15"]),
    ];
    if open >= 2 && body[open - 2..open] == inspect {
        ensure!(
            report >= 2 && body[report - 2..report] == react,
            "partial report staging"
        );
        return Ok(());
    }
    ensure!(
        open > 0
            && report > 0
            && body[open - 1] == c("iffalse", &["MnBemidjiNeedReading"])
            && body[report - 1] == c("closetext", &[]),
        "unexpected report scene preimage"
    );
    body.splice(report..report, react);
    body.splice(open..open, inspect);
    Ok(())
}
