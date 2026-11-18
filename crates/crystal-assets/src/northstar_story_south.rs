//! Stage the southern investigation and preserve independent routes to its evidence.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, MapModule};

const RANGER_REPORT: &str = "EVENT_MN_BRAINERD_REPORT";
const SURF_RECEIVED: &str = "EVENT_MN_SURF_GIFT_RECEIVED";
const RING_COPIED: &str = "EVENT_MN_DISPATCH_NOTE";

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let maps = &mut pack.data.maps;
    let clinic = maps
        .get_mut("MnRochesterPokecenter1F")
        .context("Rochester clinic")?;
    replace_sequence(
        clinic,
        "MnArcRCBattle",
        &[c(
            "winlosstext",
            &["MnArcRCWonText", "MnArcRCBattleLostText"],
        )],
        vec![c(
            "winlosstext",
            &["MnFlowReedClinicConcedesText", "MnArcRCBattleLostText"],
        )],
    )?;
    let garden = maps.get_mut("MnMankato").context("Mankato notebook")?;
    replace_sequence(
        garden,
        "MN_MANKATO_NOTEBOOK",
        &[
            c("checkflag", &["ENGINE_ZEPHYRBADGE"]),
            c("iffalse", &["MnMKNotebookDenied"]),
        ],
        vec![],
    )?;
    ellis(
        maps.get_mut("MnMinneapolisDispatchOffice")
            .context("Ellis's office")?,
    )?;
    let gym = maps.get_mut("MnMinneapolisGym").context("Avrin's Gym")?;
    put(
        gym,
        "MnMinneapolisGymRematchBeforeChampion",
        vec![
            c("checkevent", &[RING_COPIED]),
            c("iftrue", &["MnFlowAvrinAfterRing"]),
            c("jumptext", &["MnMinneapolisGymLeaderAfterText"]),
        ],
    );
    put(
        gym,
        "MnFlowAvrinAfterRing",
        vec![c("jumptext", &["MnFlowAvrinAfterRingText"])],
    );
    mina(maps.get_mut("MnStPaulUnionDepot").context("Mina's depot")?)?;
    brainerd(maps.get_mut("MnBrainerd").context("Brainerd pier")?)?;
    let station = maps
        .get_mut("MnBrainerdRangerStation")
        .context("ranger station")?;
    put(
        station,
        "MN_BRAINERD_ARCHIVIST",
        vec![
            c("checkevent", &[RANGER_REPORT]),
            c("iftrue", &["MnFlowArchivistRope"]),
            c("checkflag", &["ENGINE_MINERALBADGE"]),
            c("iffalse", &["MnFlowArchivistRope"]),
            c("jumptextfaceplayer", &["MnFlowArchivistPierText"]),
        ],
    );
    put(
        station,
        "MnFlowArchivistRope",
        vec![c("jumptextfaceplayer", &["MN_BRAINERD_ARCHIVISTText"])],
    );
    itasca(maps.get_mut("MnLakeItasca").context("Itasca reading")?);
    Ok(())
}

fn replace_sequence(
    map: &mut MapModule,
    label: &str,
    previous: &[Value],
    new: Vec<Value>,
) -> Result<()> {
    let script = map
        .scripts
        .get_mut(label)
        .and_then(Value::as_array_mut)
        .with_context(|| format!("{}/{label} script", map.id))?;
    let start = script
        .windows(previous.len())
        .position(|rows| rows == previous)
        .with_context(|| format!("expected story commands in {}/{label}", map.id))?;
    script.splice(start..start + previous.len(), new);
    Ok(())
}

fn ellis(map: &mut MapModule) -> Result<()> {
    // Pause on the existing clear route at (11, 11), facing the desk at (9, 11).
    // The player can sketch the ring at that desk on this visit or after returning.
    let path = map
        .scripts
        .get("MnMPCaptiveLeaves")
        .and_then(Value::as_array)
        .context("Ellis's departure")?;
    let mut to_desk = vec![c("step", &["DOWN"]); 2];
    to_desk.extend(vec![c("step", &["LEFT"]); 4]);
    to_desk.extend(vec![c("step", &["DOWN"]); 4]);
    let rest = path
        .strip_prefix(to_desk.as_slice())
        .context("Ellis passes his desk")?
        .to_vec();
    ensure!(
        rest.last() == Some(&c("step_end", &[])),
        "Ellis must reach the office exit"
    );
    to_desk.push(c("step_end", &[]));
    put(map, "MnFlowEllisPassesDesk", to_desk);
    put(map, "MnFlowEllisGoesHome", rest);
    let actor = "MN_ICE_CAPTIVE_DISPATCHER";
    replace_sequence(
        map,
        "MnMPCellLatch",
        &[c("applymovement", &[actor, "MnMPCaptiveLeaves"])],
        vec![
            c("applymovement", &[actor, "MnFlowEllisPassesDesk"]),
            c("turnobject", &[actor, "LEFT"]),
            c("opentext", &[]),
            c("writetext", &["MnFlowEllisAtDeskText"]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("applymovement", &[actor, "MnFlowEllisGoesHome"]),
        ],
    )
}

fn mina(map: &mut MapModule) -> Result<()> {
    // The broken-ring sketch earns the conversation. The courier's battle,
    // Reed's battle, and accepting Mina's request retain independent results.
    replace_sequence(
        map,
        "MN_ST_PAUL_DEPOT_COURIER",
        &[
            c("checkflag", &["ENGINE_PLAINBADGE"]),
            c("iffalse", &["MnSPCourierBadge"]),
        ],
        vec![],
    )?;
    replace_sequence(
        map,
        "MnArcSPBattle",
        &[c(
            "winlosstext",
            &["MnArcSPWonText", "MnArcSPBattleLostText"],
        )],
        vec![c(
            "winlosstext",
            &["MnFlowReedDepotConcedesText", "MnArcSPBattleLostText"],
        )],
    )?;
    let offer = map
        .scripts
        .get("MN_NSTP_MINA")
        .and_then(Value::as_array)
        .context("Mina's quest offer")?;
    let start = offer
        .iter()
        .position(|r| r == &c("opentext", &[]))
        .context("Mina opens her own conversation")?;
    let offer = offer[start..].to_vec();
    replace_sequence(
        map,
        "MN_NSTP_MINA",
        &offer,
        vec![c("sjump", &["MnFlowMinaOfferDate"])],
    )?;
    put(map, "MnFlowMinaOfferDate", offer);
    replace_sequence(
        map,
        "MnArcSPBattle",
        &[c("disappear", &["MN_ARC_SP_REED"]), c("end", &[])],
        vec![
            c("disappear", &["MN_ARC_SP_REED"]),
            c("faceobject", &["MN_NSTP_MINA", "PLAYER"]),
            c("sjump", &["MnFlowMinaOfferDate"]),
        ],
    )
}

fn brainerd(map: &mut MapModule) -> Result<()> {
    // Hearing the ranger is independent of making room for the optional HM.
    let speaker = "MN_BRAINERD_RANGER";
    let rows = map
        .scripts
        .get(speaker)
        .and_then(Value::as_array)
        .context("Brainerd ranger script")?;
    let start = rows
        .iter()
        .position(|r| r == &c("checkitem", &["HM_SURF"]))
        .context("Brainerd Surf offer")?;
    let mut gift = rows[start..].to_vec();
    gift.retain(|row| row != &c("setevent", &[RANGER_REPORT]));
    put(map, "MnFlowRangerOfferSurf", gift);
    put(
        map,
        speaker,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[SURF_RECEIVED]),
            c("iftrue", &["MnBrainerdRangerRepeat"]),
            c("checkflag", &["ENGINE_MINERALBADGE"]),
            c("iffalse", &["MnBrainerdRangerBefore"]),
            c("checkevent", &[RANGER_REPORT]),
            c("iftrue", &["MnFlowRangerOfferSurf"]),
            c("opentext", &[]),
            c("writetext", &["MN_BRAINERD_RANGERText"]),
            c("promptbutton", &[]),
            c("setevent", &[RANGER_REPORT]),
            c("closetext", &[]),
            c("sjump", &["MnFlowRangerOfferSurf"]),
        ],
    );
    Ok(())
}

fn itasca(map: &mut MapModule) {
    put(
        map,
        "MN_ITASCA_MARKER",
        vec![
            c("checkevent", &["EVENT_MN_HEADWATERS_READING"]),
            c("iftrue", &["MnItascaMarkerAfter"]),
            c("opentext", &[]),
            c("checkevent", &[RANGER_REPORT]),
            c("iftrue", &["MnFlowItascaRangerDate"]),
            c("writetext", &["MnFlowItascaOwnDateText"]),
            c("sjump", &["MnFlowItascaCopy"]),
        ],
    );
    put(
        map,
        "MnFlowItascaRangerDate",
        vec![
            c("writetext", &["MN_ITASCA_MARKERText"]),
            c("sjump", &["MnFlowItascaCopy"]),
        ],
    );
    put(
        map,
        "MnFlowItascaCopy",
        vec![
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("setevent", &["EVENT_MN_HEADWATERS_READING"]),
            c("jumptext", &["MnItascaMarkerAfterText"]),
        ],
    );
}
