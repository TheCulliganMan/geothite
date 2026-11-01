//! Personal aftermath, relocated characters and unique rival identity.
use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{self as arc, ArcSource, RESTORED};
use anyhow::{Context, Result};

pub(super) fn install(pack: &mut CompiledGamePack, source: &ArcSource) -> Result<()> {
    for (id, name) in [
        ("MN_LEAGUE_4", "HALDEN"),
        ("MN_R08_T07", "BRAM"),
        ("MN_R11_T05", "NILS"),
        ("MN_R16_T05", "OTTO"),
        ("MN_R17_T03", "ELI"),
    ] {
        pack.data
            .trainers
            .trainers
            .get_mut(id)
            .with_context(|| format!("old duplicate name {id}"))?
            .name = name.into();
    }
    let mut league = arc::map(pack, "MnLeagueRoom4")?;
    crate::grand_marais_modpack::authoring::text(
        &mut league,
        "MnLeagueRoom4BeforeText",
        &["HALDEN. Power needs direction. Show me how you guide yours."],
    );
    arc::publish(pack, league)?;
    let mut station = arc::map(pack, "MnStCloudStation")?;
    arc::texts(&mut station, source, "MnArcAftermath");
    arc::actor(
        pack,
        &mut station,
        "MN_ARC_AFTERMATH_REED",
        "SPRITE_RIVAL",
        [15, 16],
    )?;
    arc::jump(&mut station, "MN_ARC_AFTERMATH_REED", "MnArcAftermathText");
    arc::put(
        &mut station,
        "MnArcAftermathActor",
        vec![
            c("checkevent", &[RESTORED]),
            c("iffalse", &["MnArcAftermathHide"]),
            c("appear", &["MN_ARC_AFTERMATH_REED"]),
            c("endcallback", &[]),
        ],
    );
    arc::put(
        &mut station,
        "MnArcAftermathHide",
        vec![
            c("disappear", &["MN_ARC_AFTERMATH_REED"]),
            c("endcallback", &[]),
        ],
    );
    a::callback(&mut station, "MAPCALLBACK_OBJECTS", "MnArcAftermathActor");
    arc::publish(pack, station)?;
    let mut clinic = arc::map(pack, "MnRochesterPokecenter1F")?;
    arc::texts(&mut clinic, source, "MnArcHabitat");
    let ivy = clinic
        .objects
        .iter_mut()
        .find(|o| o.script == "MN_RC_PATIENT")
        .context("clinic patient")?;
    ivy.sprite = "SPRITE_TEACHER".into();
    ivy.event_flag = "EVENT_MN_LIGHTHOUSE_SIGNAL".into();
    arc::jump(&mut clinic, "MN_RC_PATIENT", "MnArcHabitatText");
    arc::actor(
        pack,
        &mut clinic,
        "MN_ARC_GILLPUP",
        "SPRITE_GILLPUP",
        [7, 7],
    )?;
    clinic
        .objects
        .last_mut()
        .context("displaced Gillpup")?
        .event_flag = RESTORED.into();
    arc::jump(&mut clinic, "MN_ARC_GILLPUP", "MnArcHabitatText");
    arc::publish(pack, clinic)?;
    for (map, root, prefix, label) in [
        (
            "MnWinonaFieldStation",
            "MN_W_TAMARACK",
            "MnArcTamarack",
            "MnArcTamarackAfterText",
        ),
        (
            "MnMinneapolisDispatcherHouse",
            "MN_MP_RESCUED_DISPATCHER",
            "MnArcEllis",
            "MnArcEllisAfterText",
        ),
        (
            "MnMinneapolisDispatcherHouse",
            "MN_MP_DISPATCHER_PARTNER",
            "MnArcJo",
            "MnArcJoAfterText",
        ),
        (
            "MnStPaulUnionDepot",
            "MN_NSTP_MINA",
            "MnArcMina",
            "MnArcMinaAfterText",
        ),
        (
            "MnR09Cave1F",
            "MN_NDRILL_NELL",
            "MnArcNell",
            "MnArcNellAfterText",
        ),
    ] {
        let mut room = arc::map(pack, map)?;
        arc::texts(&mut room, source, prefix);
        let target = format!("{root}ArcReturn");
        arc::jump(&mut room, &target, label);
        room.scripts
            .get_mut(root)
            .and_then(serde_json::Value::as_array_mut)
            .context("return character")?
            .splice(
                0..0,
                [c("checkevent", &[RESTORED]), c("iftrue", &[&target])],
            );
        arc::publish(pack, room)?;
    }
    Ok(())
}
