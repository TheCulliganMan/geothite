//! Repair the lighthouse before attempting its optional thank-you gift.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;

const MANIFEST: &str = "northstar-lighthouse-reward-v1";
const SIGNAL: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";
const PENDING: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL_REWARD_PENDING";
const REWARD_TEXT: &str = "MnGrandMaraisSignalRewardText";

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    texts: &BTreeMap<String, Vec<AuthoredDialoguePage>>,
) -> Result<()> {
    let Some(pages) = texts.get(REWARD_TEXT) else {
        // Historical writing snapshots retain their original quest contract.
        return Ok(());
    };
    let full_pages = texts
        .get("MnGrandMaraisSignalRewardFullText")
        .context("held lighthouse gift reminder")?;
    let mut room = pack
        .data
        .maps
        .get("MnGrandMaraisLighthouse")
        .context("installed lighthouse")?
        .clone();
    let legacy = json!([
        c("checkevent", &[SIGNAL]),
        c("iftrue", &["MnGrandMaraisSignalDone"]),
        c("checkitem", &["MACHINE_PART"]),
        c("iffalse", &["MnGrandMaraisSignalMissing"]),
        c("giveitem", &["SUPER_POTION", "1"]),
        c("iffalse", &["MnGrandMaraisSignalBagFull"]),
        c("takeitem", &["MACHINE_PART"]),
        c("setevent", &[SIGNAL]),
        c("jumptextfaceplayer", &["MnGrandMaraisSignalFixedText"])
    ]);
    let repaired = json!([
        c("checkevent", &[SIGNAL]),
        c("iftrue", &["MnGrandMaraisSignalDone"]),
        c("checkitem", &["MACHINE_PART"]),
        c("iffalse", &["MnGrandMaraisSignalMissing"]),
        c("takeitem", &["MACHINE_PART"]),
        c("setevent", &[SIGNAL]),
        c("giveitem", &["SUPER_POTION", "1"]),
        c("iffalse", &["MnGrandMaraisSignalBagFull"]),
        c("jumptextfaceplayer", &["MnGrandMaraisSignalFixedText"])
    ]);
    if pack.report.manifests.iter().any(|m| m == MANIFEST) {
        ensure!(
            room.scripts.get("MnGrandMaraisKeeper") == Some(&repaired),
            "changed lighthouse repair program"
        );
        ensure!(
            room.scripts.contains_key("MnGrandMaraisSignalRewardFull")
                && room.scripts["MnGrandMaraisSignalReward"]
                    .as_array()
                    .is_some_and(|body| body.get(1)
                        == Some(&c("iffalse", &["MnGrandMaraisSignalRewardFull"]))),
            "changed held lighthouse gift program"
        );
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(PENDING),
            "missing pending lighthouse reward flag"
        );
        return Ok(());
    }
    ensure!(
        room.scripts.get("MnGrandMaraisKeeper") == Some(&legacy),
        "changed lighthouse delivery preimage"
    );
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .contains_key(PENDING),
        "occupied pending lighthouse reward flag"
    );
    let next_flag = pack
        .data
        .story_event_script_constants
        .global
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("lighthouse event flag overflow")?;
    pack.data
        .story_event_script_constants
        .global
        .insert(PENDING.into(), next_flag);
    room.scripts.insert("MnGrandMaraisKeeper".into(), repaired);
    room.scripts.insert(
        "MnGrandMaraisSignalDone".into(),
        json!([
            c("checkevent", &[PENDING]),
            c("iftrue", &["MnGrandMaraisSignalReward"]),
            c("jumptextfaceplayer", &["MnGrandMaraisSignalDoneText"])
        ]),
    );
    room.scripts.insert(
        "MnGrandMaraisSignalBagFull".into(),
        json!([
            c("setevent", &[PENDING]),
            c("jumptextfaceplayer", &["MnGrandMaraisSignalBagFullText"])
        ]),
    );
    room.scripts.insert(
        "MnGrandMaraisSignalReward".into(),
        json!([
            c("giveitem", &["SUPER_POTION", "1"]),
            c("iffalse", &["MnGrandMaraisSignalRewardFull"]),
            c("clearevent", &[PENDING]),
            c("jumptextfaceplayer", &[REWARD_TEXT])
        ]),
    );
    room.scripts.insert(
        "MnGrandMaraisSignalRewardFull".into(),
        json!([c(
            "jumptextfaceplayer",
            &["MnGrandMaraisSignalRewardFullText"]
        )]),
    );
    room.scripts.insert(
        "MnGrandMaraisSignalRewardFullText".into(),
        authored_dialogue::compile(full_pages)?,
    );
    room.scripts
        .insert(REWARD_TEXT.into(), authored_dialogue::compile(pages)?);
    crate::northstar_story_revision::publish(pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}
