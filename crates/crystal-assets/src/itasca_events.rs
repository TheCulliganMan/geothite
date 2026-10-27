//! MS06 and MS07 complete only after their own acknowledged native text pages.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(crate) const READING: &str = "EVENT_MN_HEADWATERS_READING";
pub(crate) const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
pub(crate) fn allocate(pack: &mut CompiledGamePack) -> Result<()> {
    let flags = &mut pack.data.story_event_script_constants.global;
    ensure!(
        flags.contains_key(REPORT) && !flags.contains_key(READING),
        "existing report and unused reading identity required"
    );
    let next = flags
        .iter()
        .filter(|(n, _)| n.starts_with("EVENT_"))
        .map(|(_, v)| *v)
        .max()
        .context("event catalog")?
        .checked_add(1)
        .context("reading event overflow")?;
    u16::try_from(next).context("reading outside native event range")?;
    flags.insert(READING.into(), next);
    Ok(())
}
fn lines(
    story: &Value,
    id: &str,
    location: &str,
    requires: Value,
    flag: &str,
    pages: usize,
) -> Result<Vec<String>> {
    let matching = story["milestones"]
        .as_array()
        .context("milestones")?
        .iter()
        .filter(|m| m["id"] == id)
        .collect::<Vec<_>>();
    ensure!(matching.len() == 1, "one canonical {id} required");
    let m = matching[0];
    ensure!(
        m["location"] == location && m["requires"] == requires && m["sets"] == json!([flag]),
        "canonical {id} prerequisite/completion required"
    );
    let values: Vec<Vec<String>> = serde_json::from_value(m["dialogue_pages"].clone())?;
    ensure!(
        values.len() == pages
            && values.iter().all(|p| p.len() == 2
                && p.iter()
                    .all(|s| !s.is_empty() && s.is_ascii() && s.len() <= 18)),
        "native {id} two-line pages required"
    );
    Ok(values.into_iter().flatten().collect())
}
fn text(map: &mut MapModule, label: &str, lines: &[String]) {
    authoring::text(
        map,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
}
pub(crate) fn marker(map: &mut MapModule, story: &Value) -> Result<()> {
    let label = "MN_ITASCA_MARKERText";
    ensure!(
        map.scripts.contains_key(label),
        "actual marker sign required"
    );
    text(
        map,
        label,
        &lines(
            story,
            "MS06",
            "MnLakeItasca",
            json!(["BADGE_7"]),
            READING,
            1,
        )?,
    );
    authoring::text(
        map,
        "MnItascaMarkerBeforeText",
        &["Earn ORLIS's", "PINE BADGE first.", "BEMIDJI is east."],
    );
    authoring::text(
        map,
        "MnItascaMarkerAfterText",
        &["Reference copied.", "Return to BEMIDJI."],
    );
    let c = authoring::command;
    map.scripts.insert(
        "MN_ITASCA_MARKER".into(),
        json!([
            c("checkflag", &["ENGINE_GLACIERBADGE"]),
            c("iffalse", &["MnItascaMarkerBefore"]),
            c("checkevent", &[READING]),
            c("iftrue", &["MnItascaMarkerAfter"]),
            c("opentext", &[]),
            c("writetext", &[label]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[READING]),
            c("jumptext", &["MnItascaMarkerAfterText"])
        ]),
    );
    for (root, label) in [
        ("MnItascaMarkerBefore", "MnItascaMarkerBeforeText"),
        ("MnItascaMarkerAfter", "MnItascaMarkerAfterText"),
    ] {
        map.scripts
            .insert(root.into(), json!([c("jumptext", &[label])]));
    }
    Ok(())
}
pub(crate) fn officer(map: &mut MapModule, story: &Value) -> Result<()> {
    ensure!(
        map.scripts.contains_key("MN_BEMIDJI_OFFICER")
            && map.scripts.contains_key("MN_BEMIDJI_OFFICERText"),
        "existing field officer required"
    );
    text(
        map,
        "MnBemidjiComparisonText",
        &lines(
            story,
            "MS07",
            "MnBemidji",
            json!(["BADGE_7", READING]),
            REPORT,
            2,
        )?,
    );
    for (label, lines) in [
        (
            "MnBemidjiNeedPineText",
            vec!["Earn ORLIS's", "PINE BADGE first."],
        ),
        (
            "MnBemidjiNeedReadingText",
            vec![
                "Read the marker",
                "at LAKE ITASCA.",
                "Bring that copy",
                "back to me.",
            ],
        ),
        (
            "MnBemidjiReportAfterText",
            vec![
                "The IRON RANGE",
                "road is open.",
                "Find GRAND RAPIDS'",
                "survey depot.",
            ],
        ),
        (
            "MnBemidjiNotebookMatchText",
            vec!["Your MANKATO copy", "matches this log."],
        ),
    ] {
        authoring::text(map, label, &lines);
    }
    let c = authoring::command;
    map.scripts.insert(
        "MN_BEMIDJI_OFFICER".into(),
        json!([
            c("faceplayer", &[]),
            c("checkflag", &["ENGINE_GLACIERBADGE"]),
            c("iffalse", &["MnBemidjiNeedPine"]),
            c("checkevent", &[REPORT]),
            c("iftrue", &["MnBemidjiReportAfter"]),
            c("checkevent", &[READING]),
            c("iffalse", &["MnBemidjiNeedReading"]),
            c("opentext", &[]),
            c("writetext", &["MnBemidjiComparisonText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[REPORT]),
            c("checkevent", &["EVENT_MN_PRAIRIE_NOTEBOOK"]),
            c("iftrue", &["MnBemidjiNotebookMatch"]),
            c("jumptext", &["MnBemidjiReportAfterText"])
        ]),
    );
    for (root, label) in [
        ("MnBemidjiNeedPine", "MnBemidjiNeedPineText"),
        ("MnBemidjiNeedReading", "MnBemidjiNeedReadingText"),
    ] {
        // Keep the original advice reachable as part of the assignment. This
        // preserves its ordinary text in uncached native reconstruction too.
        map.scripts.insert(
            root.into(),
            json!([
                c("opentext", &[]),
                c("writetext", &["MN_BEMIDJI_OFFICERText"]),
                c("waitbutton", &[]),
                c("writetext", &[label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    map.scripts.insert(
        "MnBemidjiReportAfter".into(),
        json!([c("jumptextfaceplayer", &["MnBemidjiReportAfterText"])]),
    );
    map.scripts.insert(
        "MnBemidjiNotebookMatch".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnBemidjiNotebookMatchText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("jumptext", &["MnBemidjiReportAfterText"])
        ]),
    );
    Ok(())
}
