//! Birchit's optional adoption and independently collectible travel supplies.
#[path = "northstar_marais_station.rs"]
mod station;
#[path = "northstar_winona_aide_guidance.rs"]
mod winona;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::put;
use crate::{
    CompiledGamePack, MapModule, authored_dialogue, authored_town_publication, gym_scripts,
};

const MAP: &str = "MnGrandMaraisFieldStation";
const STARTER: &str = "EVENT_MN_GRAND_MARAIS_STARTER";
const SIGNAL: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";
const OPEN: &str = "EVENT_MN_MARAIS_SUPPLIES_OPEN";
const BALLS: &str = "EVENT_MN_MARAIS_BALLS_RECEIVED";
const POTIONS: &str = "EVENT_MN_MARAIS_POTIONS_RECEIVED";
const FIRST_OFFER: &str = "MnRewardMachinePartOriginal";
const AFTER_REPAIR: &str = "MnMaraisGiftAfterRepair";
const GIFT: &str = "MnMaraisPartnerGift";
const REPAIRED_RECEIPT: &str = "MnMaraisPartnerAfterRepair";
const RECORD: &str = "MnMaraisRecordPartner";
const BOX: &str = "MnMaraisPartnerBox";
const DECLINE: &str = "MnMaraisPartnerDeclined";
const RETURN: &str = "MnMaraisSupplyReturn";
const CLAIM: &str = "MnMaraisTravelSupplies";
const MEDICINE: &str = "MnMaraisTravelMedicine";
const CHECK: &str = "MnMaraisCheckSupplies";
const CHECK_BALLS: &str = "MnMaraisCheckWaitingBalls";
const HOLD_POTIONS: &str = "MnMaraisHoldPotions";
const HOLD_BOTH: &str = "MnMaraisHoldBoth";
const TEXTS: [&str; 7] = [
    "MnMaraisPartnerDeclinedText",
    "MnMaraisPartnerBoxText",
    "MnMaraisBallsReceivedText",
    "MnMaraisPotionsReceivedText",
    "MnMaraisBallsWaitingText",
    "MnMaraisPotionsWaitingText",
    "MnMaraisBothWaitingText",
];

fn consent(room: &mut MapModule) -> Result<()> {
    for label in [FIRST_OFFER, AFTER_REPAIR] {
        let offer = room
            .scripts
            .get_mut(label)
            .and_then(Value::as_array_mut)
            .context(label)?;
        let old = [
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &[GIFT]),
        ];
        ensure!(
            offer.ends_with(&old),
            "unexpected Birchit invitation in {label}"
        );
        offer.truncate(offer.len() - old.len());
        offer.extend([
            c("yesorno", &[]),
            c("iffalse", &[DECLINE]),
            c("closetext", &[]),
            c("sjump", &[GIFT]),
        ]);
        if label == FIRST_OFFER {
            ensure!(
                offer.starts_with(&[
                    c("checkevent", &[STARTER]),
                    c("iftrue", &["MnGrandMaraisWelcomeAfter"])
                ]),
                "missing completed gift guard"
            );
            // Saves with a completed legacy adoption have no outstanding modern
            // delivery. Preserve their original conversation and rewards.
            offer.splice(0..0, [c("checkevent", &[OPEN]), c("iftrue", &[RETURN])]);
        }
    }
    put(room, DECLINE, close_with("MnMaraisPartnerDeclinedText"));

    let gift = room
        .scripts
        .get_mut(GIFT)
        .and_then(Value::as_array_mut)
        .context("Birchit handoff")?;
    let old = [
        c("givepoke", &["BIRCHIT", "8", "BERRY"]),
        c("ifequal", &["2", "MnMaraisPartnerFull"]),
        c("giveitem", &["POKE_BALL", "5"]),
        c("giveitem", &["POTION", "3"]),
        c("setevent", &[STARTER]),
        c("setscene", &["SCENE_MN_FIELD_READY"]),
        c("checkevent", &[SIGNAL]),
        c("iftrue", &[REPAIRED_RECEIPT]),
    ];
    ensure!(
        gift.starts_with(&old)
            && gift.ends_with(&[c("promptbutton", &[]), c("closetext", &[]), c("end", &[])]),
        "unexpected Birchit delivery"
    );
    gift.splice(2..6, [c("ifequal", &["1", BOX]), c("scall", &[RECORD])]);
    gift.truncate(gift.len() - 2);
    gift.push(c("sjump", &[CLAIM]));
    let repaired = room
        .scripts
        .get_mut(REPAIRED_RECEIPT)
        .context(REPAIRED_RECEIPT)?;
    ensure!(
        *repaired
            == json!([
                c("opentext", &[]),
                c("writetext", &["MnMaraisPartnerAfterRepairText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        "unexpected repaired-harbor receipt"
    );
    *repaired = json!([
        c("opentext", &[]),
        c("writetext", &["MnMaraisPartnerAfterRepairText"]),
        c("promptbutton", &[]),
        c("sjump", &[CLAIM])
    ]);
    put(
        room,
        RECORD,
        vec![
            c("setevent", &[STARTER]),
            c("setscene", &["SCENE_MN_FIELD_READY"]),
            c("setevent", &[OPEN]),
            c("end", &[]),
        ],
    );
    put(
        room,
        BOX,
        vec![
            c("scall", &[RECORD]),
            c("opentext", &[]),
            c("writetext", &["MnMaraisPartnerBoxText"]),
            c("promptbutton", &[]),
            c("sjump", &[CLAIM]),
        ],
    );
    Ok(())
}

fn close_with(label: &str) -> Vec<Value> {
    vec![
        c("writetext", &[label]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[]),
    ]
}

fn supplies(room: &mut MapModule) {
    put(room, RETURN, vec![c("opentext", &[]), c("sjump", &[CLAIM])]);
    for (label, receipt, item, amount, notice, next) in [
        (
            CLAIM,
            BALLS,
            "POKE_BALL",
            "5",
            "MnMaraisBallsReceivedText",
            MEDICINE,
        ),
        (
            MEDICINE,
            POTIONS,
            "POTION",
            "3",
            "MnMaraisPotionsReceivedText",
            CHECK,
        ),
    ] {
        put(
            room,
            label,
            vec![
                c("checkevent", &[receipt]),
                c("iftrue", &[next]),
                c("giveitem", &[item, amount]),
                c("iffalse", &[next]),
                c("setevent", &[receipt]),
                c("writetext", &[notice]),
                c("promptbutton", &[]),
                c("sjump", &[next]),
            ],
        );
    }
    put(
        room,
        CHECK,
        vec![
            c("checkevent", &[BALLS]),
            c("iffalse", &[CHECK_BALLS]),
            c("checkevent", &[POTIONS]),
            c("iffalse", &[HOLD_POTIONS]),
            c("clearevent", &[OPEN]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    let mut balls = vec![c("checkevent", &[POTIONS]), c("iffalse", &[HOLD_BOTH])];
    balls.extend(close_with("MnMaraisBallsWaitingText"));
    put(room, CHECK_BALLS, balls);
    put(room, HOLD_POTIONS, close_with("MnMaraisPotionsWaitingText"));
    put(room, HOLD_BOTH, close_with("MnMaraisBothWaitingText"));
}

/// Adds a consent prompt to both Marais offers and lets the player collect missed supplies.
///
/// # Errors
/// Returns an error for an unexpected parent or handoff, occupied labels or flags,
/// or an invalid compiled archive.
pub fn revise_marais_supplies(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-ely-return-v1"),
        "Marais supplies require Ely Return"
    );
    let mut room = pack.data.maps.get(MAP).context(MAP)?.clone();
    for label in [
        station::DOORWAY,
        RECORD,
        BOX,
        DECLINE,
        RETURN,
        CLAIM,
        MEDICINE,
        CHECK,
        CHECK_BALLS,
        HOLD_POTIONS,
        HOLD_BOTH,
    ]
    .into_iter()
    .chain(TEXTS)
    {
        ensure!(
            !room.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "occupied field station label {label}"
        );
    }
    for flag in [OPEN, BALLS, POTIONS] {
        ensure!(
            !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(flag),
            "occupied field station flag {flag}"
        );
        gym_scripts::event(&mut pack, flag)?;
    }
    consent(&mut room)?;
    supplies(&mut room);
    for label in TEXTS {
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    station::apply(&mut room)?;
    room.script_control_commands = crate::parse_script_control_commands(MAP, &room.scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(MAP, &room.scripts)?;
    room.script_scene_commands = crate::parse_script_scene_commands(MAP, &room.scripts)?;
    room.script_item_grants = crate::parse_script_item_grants(MAP, &room.scripts)?;
    (room.script_item_checks, room.script_item_takes) =
        crate::parse_script_item_accesses(MAP, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(MAP, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(MAP, &room.scripts)?;
    authored_town_publication::replace(&mut pack, room)?;
    winona::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-marais-supplies-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
