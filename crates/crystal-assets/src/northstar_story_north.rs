//! Follow the mine evidence into the rescue and give the crew a lasting aftermath.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";

/// Revises the selected northern scripts before their runtime mirrors are rebuilt.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let maps = &mut pack.data.maps;
    revise_harbor(
        maps.get_mut("MnDuluthHarborWorkshop")
            .context("Duluth harbor workshop")?,
    )?;
    revise_hale(
        maps.get_mut("MnBoundaryWatersSurveyShelter")
            .context("Boundary survey shelter")?,
    )?;
    revise_intake(
        maps.get_mut("MnBoundaryIntake")
            .context("Boundary intake")?,
    )
}

fn revise_harbor(room: &mut MapModule) -> Result<()> {
    let receiver = room
        .scripts
        .get_mut("MN_HARBOR_RECEIVER")
        .context("harbor evidence receiver")?;
    *receiver = json!([
        command("faceplayer", &[]),
        command("checkevent", &[SIGNAL]),
        command("iftrue", &["MN_HARBOR_RECEIVERDone"]),
        command("checkevent", &["EVENT_MN_DEPOT_CLEARED"]),
        command("iffalse", &["MnOverhaulHarborMissingLog"]),
        command("opentext", &[]),
        command("writetext", &["MN_HARBOR_RECEIVERText"]),
        command("promptbutton", &[]),
        command("closetext", &[]),
        command("pause", &["10"]),
        command("opentext", &[]),
        command("writetext", &["MnArcHarborIntakeText"]),
        command("promptbutton", &[]),
        command("writetext", &["MnOverhaulHarborDirectionsText"]),
        command("promptbutton", &[]),
        command("setevent", &[SIGNAL]),
        command("closetext", &[]),
        command("end", &[])
    ]);
    let repeat = room
        .scripts
        .get_mut("MN_HARBOR_RECEIVERDone")
        .context("harbor signal return")?;
    *repeat = json!([
        command("checkevent", &[RESTORED]),
        command("iftrue", &["MnHarborSurveyReturn"]),
        command("jumptextfaceplayer", &["MN_HARBOR_RECEIVERDoneText"])
    ]);
    Ok(())
}

fn revise_hale(room: &mut MapModule) -> Result<()> {
    let chief = room
        .scripts
        .get_mut("MN_SURVEY_CHIEF")
        .and_then(Value::as_array_mut)
        .context("Hale's survey shelter script")?;
    // The signal is earned by taking Finn's meter to the depot and its times
    // to the harbor. Keep the battle, victory receipt and return path intact.
    let badges: Vec<_> = crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8]
        .iter()
        .flat_map(|flag| {
            [
                command("checkflag", &[flag]),
                command("iffalse", &["MnBoundarySurveyChiefBadgeDenied"]),
            ]
        })
        .collect();
    let first = chief
        .windows(badges.len())
        .position(|rows| rows == badges.as_slice())
        .context("Hale's eight sporting checks")?;
    ensure!(
        chief.get(first + badges.len()) == Some(&command("checkevent", &[SIGNAL])),
        "the chief must require the harbor signal"
    );
    chief.drain(first..first + badges.len());
    Ok(())
}

fn revise_intake(room: &mut MapModule) -> Result<()> {
    let finn = room
        .scripts
        .get_mut("MN_ARC_INTAKE_FINN")
        .context("Finn on the intake steps")?;
    *finn = json!([
        command("faceplayer", &[]),
        command("checkevent", &[RESTORED]),
        command("iftrue", &["MnFlowFinnRestored"]),
        command("checkevent", &["EVENT_MN_VERDIGRIM_OUTLET_CLOSED"]),
        command("iftrue", &["MnFlowFinnBypass"]),
        command("checkevent", &["EVENT_MN_VERDIGRIM_DRILL_STOPPED"]),
        command("iftrue", &["MnFlowFinnOutlet"]),
        command("jumptext", &["MnArcIntakeFinnText"])
    ]);
    for (branch, text) in [
        ("MnFlowFinnRestored", "MnFlowFinnRestoredText"),
        ("MnFlowFinnBypass", "MnFlowFinnBypassText"),
        ("MnFlowFinnOutlet", "MnFlowFinnOutletText"),
    ] {
        room.scripts
            .insert(branch.into(), json!([command("jumptext", &[text])]));
    }

    let battle = room
        .scripts
        .get_mut("MnArcFinalBattle")
        .and_then(Value::as_array_mut)
        .context("Reed's intake battle")?;
    let handoff = battle
        .iter_mut()
        .find(|row| **row == command("writetext", &["MnArcFinalWonText"]))
        .context("post-battle access to the drill")?;
    *handoff = command("writetext", &["MnFlowFinalDriveClearText"]);

    let bypass = room
        .scripts
        .get_mut("MnVerdigrimBypass")
        .and_then(Value::as_array_mut)
        .context("cage and crew rescue")?;
    let speech = [
        command("opentext", &[]),
        command("writetext", &["MnArcIntakeReedExitText"]),
        command("waitbutton", &[]),
        command("closetext", &[]),
    ];
    let start = bypass
        .windows(speech.len())
        .position(|rows| rows == speech)
        .context("Reed's pay demand during the crew rescue")?;
    bypass.drain(start..start + speech.len());
    Ok(())
}
