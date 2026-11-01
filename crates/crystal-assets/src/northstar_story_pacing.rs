//! Accept evidence promptly while retaining the region's travel safeguards.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::{LINK, RESTORED};
use crate::{CompiledGamePack, MapModule};

pub(crate) const MANIFEST: &str = "northstar-story-pacing-v1";

/// Revises only the reviewed investigation and completed-chief branches.
/// Existing flags retain their identities; road and finale access guards remain.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|m| m == MANIFEST),
        "investigation pacing already installed"
    );
    let mut park = pack.data.maps["MnLakeItasca"].clone();
    remove_badge_gate(&mut park, "MN_ITASCA_MARKER", 0, "MnItascaMarkerBefore")?;
    crate::northstar_story_revision::publish(pack, park)?;

    let mut officer = pack.data.maps["MnBemidji"].clone();
    remove_badge_gate(&mut officer, "MN_BEMIDJI_OFFICER", 1, "MnBemidjiNeedPine")?;
    crate::northstar_story_revision::publish(pack, officer)?;

    harbor(pack)?;
    chief(pack)?;
    returns(pack)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn returns(pack: &mut CompiledGamePack) -> Result<()> {
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
        let mut map = pack.data.maps[name].clone();
        ensure!(
            map.scripts.get(root) == Some(&serde_json::json!([c(command, &[original])])),
            "changed investigation return preimage: {name}/{root}"
        );
        let mut body = Vec::new();
        for (event, text) in branches {
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

fn remove_badge_gate(map: &mut MapModule, script: &str, at: usize, branch: &str) -> Result<()> {
    let body = map
        .scripts
        .get_mut(script)
        .and_then(Value::as_array_mut)
        .context("investigation program")?;
    ensure!(
        body.get(at) == Some(&c("checkflag", &["ENGINE_GLACIERBADGE"]))
            && body.get(at + 1) == Some(&c("iffalse", &[branch])),
        "changed investigation badge preimage: {script}"
    );
    body.drain(at..at + 2);
    Ok(())
}

fn harbor(pack: &mut CompiledGamePack) -> Result<()> {
    let mut map = pack.data.maps["MnDuluthHarborWorkshop"].clone();
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
    ensure!(
        body.get(3..19) == Some(expected.as_slice()),
        "changed harbor badge gates"
    );
    ensure!(
        body.get(19) == Some(&c("checkevent", &["EVENT_MN_DEPOT_CLEARED"])),
        "harbor must still require the recovered pump log"
    );
    body.drain(3..19);

    // A crew accepts the log and starts planning before the final Gym. Its
    // later advice can distinguish travel preparation from missing evidence.
    let done = map
        .scripts
        .get_mut("MN_HARBOR_RECEIVERDone")
        .and_then(Value::as_array_mut)
        .context("harbor return program")?;
    ensure!(
        done.first() == Some(&c("checkevent", &[RESTORED]))
            && done.get(1) == Some(&c("iftrue", &["MnHarborSurveyReturn"]))
            && done.get(2) == Some(&c("jumptextfaceplayer", &["MN_HARBOR_RECEIVERDoneText"])),
        "changed completed-action harbor branch"
    );
    done.splice(
        2..2,
        [
            c("checkflag", &["ENGINE_RISINGBADGE"]),
            c("iffalse", &["MN_HARBOR_RECEIVERDenied"]),
        ],
    );
    crate::northstar_story_revision::publish(pack, map)
}

fn chief(pack: &mut CompiledGamePack) -> Result<()> {
    let mut map = pack.data.maps["MnBoundaryWatersSurveyShelter"].clone();
    let label = "MnBoundarySurveyChiefAfter";
    ensure!(
        map.scripts.get(label)
            == Some(&serde_json::json!([c(
                "jumptextfaceplayer",
                &["MnBoundarySurveyChiefAfterText"]
            )])),
        "changed defeated-chief return program"
    );
    map.scripts.insert(
        label.into(),
        serde_json::json!([
            c("checkevent", &[RESTORED]),
            c("iftrue", &["MnOverhaulChiefStopped"]),
            c("checkevent", &[LINK]),
            c("iftrue", &["MnOverhaulChiefExposed"]),
            c("jumptextfaceplayer", &["MnBoundarySurveyChiefAfterText"])
        ]),
    );
    for (script, text) in [
        ("MnOverhaulChiefStopped", "MnOverhaulChiefStoppedText"),
        ("MnOverhaulChiefExposed", "MnOverhaulChiefExposedText"),
    ] {
        ensure!(
            map.scripts.contains_key(text),
            "canonical chief aftermath text: {text}"
        );
        map.scripts.insert(
            script.into(),
            serde_json::json!([c("jumptextfaceplayer", &[text])]),
        );
    }
    crate::northstar_story_revision::publish(pack, map)
}
