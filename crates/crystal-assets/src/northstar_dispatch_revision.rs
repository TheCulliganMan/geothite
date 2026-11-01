//! Ellis supplies directions; the office desk supplies the dispatch evidence.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command as c;

const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut home = pack
        .data
        .maps
        .get("MnMinneapolisDispatcherHouse")
        .context("Ellis's home")?
        .clone();
    let conversation = home
        .scripts
        .get_mut("MN_MP_RESCUED_DISPATCHER")
        .and_then(serde_json::Value::as_array_mut)
        .context("Ellis's dispatch directions")?;
    let text = conversation
        .iter()
        .position(|row| row == &c("writetext", &["MnNICEEllisQuestText"]))
        .context("Ellis's directions text")?;
    let tail = &conversation[text + 1..];
    let acknowledged = tail.first() == Some(&c("waitbutton", &[]))
        || tail.first() == Some(&c("promptbutton", &[]));
    ensure!(
        acknowledged
            && (tail[1..] == [c("setevent", &[NOTE]), c("closetext", &[]), c("end", &[])]
                || tail[1..] == [c("closetext", &[]), c("end", &[])]),
        "unexpected Ellis dispatch reward sequence"
    );
    conversation.truncate(text + 1);
    conversation.extend([c("promptbutton", &[]), c("closetext", &[]), c("end", &[])]);
    crate::northstar_story_revision::publish(pack, home)?;

    let mut office = pack
        .data
        .maps
        .get("MnMinneapolisDispatchOffice")
        .context("dispatch office")?
        .clone();
    let copy = json!([
        c("checkevent", &[NOTE]),
        c("iftrue", &["MnMPDispatchDone"]),
        c("checkevent", &["EVENT_MN_ICE_CAPTIVE_RESCUED"]),
        c("iffalse", &["MnMPDispatchLocked"]),
        c("checkevent", &["EVENT_MN_ICE_HOLDING_POWER_OFF"]),
        c("iffalse", &["MnMPDispatchLocked"]),
        c("checkevent", &["EVENT_MN_ICE_ENFORCER_DEFEATED"]),
        c("iffalse", &["MnMPDispatchLocked"]),
        c("opentext", &[]),
        c("writetext", &["MnMPDispatchReadText"]),
        c("promptbutton", &[]),
        c("setevent", &[NOTE]),
        c("closetext", &[]),
        c("end", &[]),
    ]);
    let installed = office
        .scripts
        .get("MnMPDispatchCopy")
        .context("hidden dispatch copy")?;
    ensure!(
        installed == &json!([c("jumptext", &["MnNICECopyFlavorText"])]) || installed == &copy,
        "unexpected office dispatch-copy program"
    );
    ensure!(
        office
            .events
            .bg_events
            .iter()
            .any(|event| event.x == 9 && event.y == 11 && event.script == "MnMPDispatchCopy"),
        "hidden copy must remain at the office desk"
    );
    office.scripts.insert("MnMPDispatchCopy".into(), copy);
    crate::northstar_story_revision::publish(pack, office)
}
