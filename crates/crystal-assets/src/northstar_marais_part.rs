//! Deliver the urgent lighthouse part independently of the partner gift.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;

const MANIFEST: &str = "northstar-marais-part-first-v1";
const STARTER: &str = "EVENT_MN_GRAND_MARAIS_STARTER";
const SIGNAL: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    texts: &BTreeMap<String, Vec<AuthoredDialoguePage>>,
) -> Result<()> {
    if !texts.contains_key("MnMaraisGiftAfterRepairText") {
        return Ok(());
    }
    let mut room = pack
        .data
        .maps
        .get("MnGrandMaraisFieldStation")
        .context("Grand Marais field station")?
        .clone();
    let part = json!([
        c("checkevent", &[SIGNAL]),
        c("iftrue", &["MnRewardMachinePartOriginal"]),
        c("checkitem", &["MACHINE_PART"]),
        c("iftrue", &["MnRewardMachinePartOriginal"]),
        c("opentext", &[]),
        c("verbosegiveitem", &["MACHINE_PART", "1"]),
        c("iffalse", &["MnRewardMachinePartFull"]),
        c("writetext", &["MnRewardMachinePartGivenText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[])
    ]);
    if pack.report.manifests.iter().any(|m| m == MANIFEST) {
        ensure!(
            room.scripts.get("MnRewardMachinePart") == Some(&part),
            "changed lighthouse part handoff"
        );
        return Ok(());
    }
    let mut legacy_part = part.as_array().context("part command list")?.clone();
    legacy_part.splice(
        0..0,
        [
            c("checkevent", &[STARTER]),
            c("iffalse", &["MnRewardMachinePartOriginal"]),
        ],
    );
    ensure!(
        room.scripts.get("MnRewardMachinePart") == Some(&json!(legacy_part)),
        "changed legacy part handoff"
    );
    let original = room
        .scripts
        .get("MnRewardMachinePartOriginal")
        .and_then(serde_json::Value::as_array)
        .context("installed partner gift")?;
    let gift_index = original
        .iter()
        .position(|row| row == &c("givepoke", &["BIRCHIT", "8", "BERRY"]))
        .context("native Birchit gift")?;
    let mut gift = original[gift_index..].to_vec();
    ensure!(
        gift.get(1) == Some(&c("ifequal", &["2", "MnMaraisPartnerFull"])),
        "partner storage failure branch"
    );
    let part_index = gift
        .iter()
        .position(|row| row == &c("giveitem", &["MACHINE_PART", "1"]))
        .context("legacy part follows gift")?;
    gift.remove(part_index);
    let result_index = gift
        .windows(2)
        .position(|rows| {
            rows[0] == c("opentext", &[])
                && rows[1] == c("writetext", &["MnGrandMaraisPartnerText"])
        })
        .context("partner receipt")?;
    gift.splice(
        result_index..result_index,
        [
            c("checkevent", &[SIGNAL]),
            c("iftrue", &["MnMaraisPartnerAfterRepair"]),
        ],
    );
    room.scripts.insert("MnRewardMachinePart".into(), part);
    room.scripts.insert(
        "MnRewardMachinePartOriginal".into(),
        json!([
            c("checkevent", &[STARTER]),
            c("iftrue", &["MnGrandMaraisWelcomeAfter"]),
            c("checkevent", &[SIGNAL]),
            c("iftrue", &["MnMaraisGiftAfterRepair"]),
            c("opentext", &[]),
            c("writetext", &["MnGrandMaraisWelcomeText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &["MnMaraisPartnerGift"])
        ]),
    );
    room.scripts
        .insert("MnMaraisPartnerGift".into(), json!(gift));
    room.scripts.insert(
        "MnMaraisGiftAfterRepair".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnMaraisGiftAfterRepairText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &["MnMaraisPartnerGift"])
        ]),
    );
    room.scripts.insert(
        "MnMaraisPartnerAfterRepair".into(),
        json!([
            c("opentext", &[]),
            c("writetext", &["MnMaraisPartnerAfterRepairText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for label in [
        "MnMaraisGiftAfterRepairText",
        "MnMaraisPartnerAfterRepairText",
    ] {
        room.scripts.insert(
            label.into(),
            authored_dialogue::compile(texts.get(label).context("canonical Marais gift pages")?)?,
        );
    }
    crate::northstar_story_revision::publish(pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}
