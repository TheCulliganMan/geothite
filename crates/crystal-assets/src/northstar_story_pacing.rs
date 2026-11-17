//! Accept evidence promptly while retaining the region's travel safeguards.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::{LINK, RESTORED};

pub(crate) const MANIFEST: &str = "northstar-story-pacing-v1";

/// Revises only the reviewed investigation and completed-chief branches.
/// Existing flags retain their identities; road and finale access guards remain.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|m| m == MANIFEST),
        "investigation pacing already installed"
    );
    install_headwaters(pack)?;
    install_harbor(pack)?;
    install_returns(pack, None)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

pub(crate) fn install_returns(
    pack: &mut CompiledGamePack,
    reviewed_texts: Option<&BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>>,
) -> Result<()> {
    const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
    const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
    const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
    const METER: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
    for (name, root, command, original, branches) in [
        (
            "MnBemidji",
            "MnBemidjiReportAfter",
            "jumptextfaceplayer",
            "MnBemidjiReportAfterText",
            vec![
                (RESTORED, "MnOverhaulOfficerDoneText"),
                (LINK, "MnOverhaulOfficerIntakeText"),
                (SIGNAL, "MnOverhaulOfficerNorthText"),
                (DEPOT, "MnOverhaulOfficerHarborText"),
                (METER, "MnOverhaulOfficerMeterText"),
            ],
        ),
        (
            "MnBoundaryWatersSurveyShelter",
            "MnBoundarySurveyChiefAfter",
            "jumptextfaceplayer",
            "MnBoundarySurveyChiefAfterText",
            vec![
                (RESTORED, "MnOverhaulChiefStoppedText"),
                (LINK, "MnOverhaulChiefExposedText"),
            ],
        ),
        (
            "MnLakeItasca",
            "MnItascaMarkerAfter",
            "jumptext",
            "MnItascaMarkerAfterText",
            vec![(REPORT, "MnOverhaulItascaComparedText")],
        ),
        (
            "MnBrainerd",
            "MnBrainerdRangerRepeat",
            "jumptextfaceplayer",
            "MnBrainerdRangerRepeatText",
            vec![(REPORT, "MnOverhaulRangerComparedText")],
        ),
        (
            "MnGrandRapidsSurveyDepot",
            "MnGrandRapidsDepotDone",
            "jumptextfaceplayer",
            "MnGrandRapidsDepotDoneText",
            vec![
                (RESTORED, "MnOverhaulDepotDoneText"),
                (LINK, "MnOverhaulDepotIntakeText"),
                (SIGNAL, "MnOverhaulDepotNorthText"),
            ],
        ),
    ] {
        let original_body = serde_json::json!([c(command, &[original])]);
        let texts = if let Some(reviewed) = reviewed_texts {
            let Some(texts) = reviewed.get(name) else {
                continue;
            };
            if branches
                .iter()
                .any(|(_, label)| !texts.contains_key(*label))
                || pack
                    .data
                    .maps
                    .get(name)
                    .and_then(|map| map.scripts.get(root))
                    != Some(&original_body)
            {
                continue;
            }
            Some(texts)
        } else {
            None
        };
        let mut map = pack
            .data
            .maps
            .get(name)
            .with_context(|| format!("return map {name}"))?
            .clone();
        ensure!(
            map.scripts.get(root) == Some(&original_body),
            "changed investigation return preimage: {name}/{root}"
        );
        let mut body = Vec::new();
        for (event, text) in branches {
            if let Some(texts) = texts {
                map.scripts
                    .insert(text.into(), authored_dialogue::compile(&texts[text])?);
            }
            ensure!(
                map.scripts.contains_key(text),
                "canonical investigation return: {text}"
            );
            let script = text.strip_suffix("Text").context("return text suffix")?;
            body.extend([c("checkevent", &[event]), c("iftrue", &[script])]);
            map.scripts
                .insert(script.into(), serde_json::json!([c(command, &[text])]));
        }
        body.push(c(command, &[original]));
        map.scripts.insert(root.into(), Value::Array(body));
        crate::northstar_story_revision::publish(pack, map)?;
    }
    Ok(())
}

pub(crate) fn install_headwaters(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, script, at, denied, event, after) in [
        (
            "MnLakeItasca",
            "MN_ITASCA_MARKER",
            0,
            "MnItascaMarkerBefore",
            "EVENT_MN_HEADWATERS_READING",
            "MnItascaMarkerAfter",
        ),
        (
            "MnBemidji",
            "MN_BEMIDJI_OFFICER",
            1,
            "MnBemidjiNeedPine",
            "EVENT_MN_HEADWATERS_REPORT",
            "MnBemidjiReportAfter",
        ),
    ] {
        let Some(mut map) = pack.data.maps.get(name).cloned() else {
            continue;
        };
        let body = map
            .scripts
            .get_mut(script)
            .and_then(Value::as_array_mut)
            .context("headwaters investigation program")?;
        let completed = [c("checkevent", &[event]), c("iftrue", &[after])];
        let remove_gate = body.get(at..at + 2) != Some(completed.as_slice());
        if remove_gate {
            let gate = [
                c("checkflag", &["ENGINE_GLACIERBADGE"]),
                c("iffalse", &[denied]),
            ];
            ensure!(
                body.get(at..at + 2) == Some(gate.as_slice())
                    && body.get(at + 2..at + 4) == Some(completed.as_slice()),
                "changed investigation badge preimage: {script}"
            );
            body.drain(at..at + 2);
        }
        let removed_denial = map.scripts.remove(denied).is_some();
        let removed_text = map.scripts.remove(&format!("{denied}Text")).is_some();
        if remove_gate || removed_denial || removed_text {
            crate::northstar_story_revision::publish(pack, map)?;
        }
    }
    Ok(())
}

pub(crate) fn install_harbor_missing_log(
    pack: &mut CompiledGamePack,
    pages: &[AuthoredDialoguePage],
) -> Result<()> {
    const MAP: &str = "MnDuluthHarborWorkshop";
    const TEXT: &str = "MnOverhaulHarborMissingLogText";
    const BRANCH: &str = "MnOverhaulHarborMissingLog";
    let Some(mut room) = pack.data.maps.get(MAP).cloned() else {
        return Ok(());
    };
    let Some(body) = room
        .scripts
        .get_mut("MN_HARBOR_RECEIVER")
        .and_then(Value::as_array_mut)
    else {
        return Ok(());
    };
    let depot_gate = [
        c("checkevent", &["EVENT_MN_DEPOT_CLEARED"]),
        c("iffalse", &["MN_HARBOR_RECEIVERDenied"]),
    ];
    let badge_gate = c("checkflag", &["ENGINE_ZEPHYRBADGE"]);
    let return_gate = [
        c("checkevent", &["EVENT_MN_LIGHTHOUSE_SIGNAL"]),
        c("iftrue", &["MN_HARBOR_RECEIVERDone"]),
    ];
    if body.get(1..3) != Some(return_gate.as_slice()) || body.get(3) != Some(&badge_gate) {
        return Ok(());
    }
    let Some(at) = body.windows(2).position(|commands| commands == depot_gate) else {
        return Ok(());
    };
    body.drain(at..at + 2);
    body.splice(3..3, [depot_gate[0].clone(), c("iffalse", &[BRANCH])]);
    room.scripts.insert(
        BRANCH.into(),
        serde_json::json!([c("jumptextfaceplayer", &[TEXT])]),
    );
    room.scripts
        .insert(TEXT.into(), authored_dialogue::compile(pages)?);
    crate::northstar_story_revision::publish(pack, room)
}

pub(crate) fn install_harbor(pack: &mut CompiledGamePack) -> Result<()> {
    let Some(mut map) = pack.data.maps.get("MnDuluthHarborWorkshop").cloned() else {
        return Ok(());
    };
    let body = map
        .scripts
        .get_mut("MN_HARBOR_RECEIVER")
        .and_then(Value::as_array_mut)
        .context("harbor evidence program")?;
    let badges = crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
    let expected = badges[..8]
        .iter()
        .flat_map(|badge| {
            [
                c("checkflag", &[badge]),
                c("iffalse", &["MN_HARBOR_RECEIVERDenied"]),
            ]
        })
        .collect::<Vec<_>>();
    let custody = [
        c("checkevent", &["EVENT_MN_DEPOT_CLEARED"]),
        c("iffalse", &["MnOverhaulHarborMissingLog"]),
    ];
    let badge_at = if body.get(3..5) == Some(custody.as_slice()) {
        5
    } else {
        ensure!(
            body.get(19..21) == Some(custody.as_slice()),
            "harbor must still require the recovered pump log"
        );
        3
    };
    let already_open = badge_at == 5 && body.get(5) == Some(&c("opentext", &[]));
    if !already_open {
        ensure!(
            body.get(badge_at..badge_at + expected.len()) == Some(expected.as_slice()),
            "changed harbor badge gates"
        );
        body.drain(badge_at..badge_at + expected.len());
    }

    // A crew accepts the log and starts planning before the final Gym. Its
    // later advice can distinguish travel preparation from missing evidence.
    let done = map
        .scripts
        .get_mut("MN_HARBOR_RECEIVERDone")
        .and_then(Value::as_array_mut)
        .context("harbor return program")?;
    ensure!(
        done.first() == Some(&c("checkevent", &[RESTORED]))
            && done.get(1) == Some(&c("iftrue", &["MnHarborSurveyReturn"])),
        "changed completed-action harbor branch"
    );
    let reminder = [
        c("checkflag", &["ENGINE_RISINGBADGE"]),
        c("iffalse", &["MN_HARBOR_RECEIVERDenied"]),
    ];
    let has_reminder = done.get(2..4) == Some(reminder.as_slice());
    ensure!(
        done.get(if has_reminder { 4 } else { 2 })
            == Some(&c("jumptextfaceplayer", &["MN_HARBOR_RECEIVERDoneText"])),
        "changed harbor return reply"
    );
    if !has_reminder {
        done.splice(2..2, reminder);
    }
    if !already_open || !has_reminder {
        crate::northstar_story_revision::publish(pack, map)?;
    }
    Ok(())
}
