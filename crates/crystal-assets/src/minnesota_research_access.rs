//! Free early research partners and both restored fossil families in one run.
use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring::flag;
use crate::northstar_rival_arc::{map, publish, put};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-early-research-access-v1";
const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";
const EXCHANGED: &str = "EVENT_MN_FOSSIL_EXCHANGED";

/// Makes the two unchosen starters free after the first Rochester delivery.
///
/// The existing Winona aide runs the original research reward program first,
/// then offers one unclaimed level-five partner per visit. Original choice and
/// prior Kindlit claims count toward completion. Refusal and a full party leave
/// each claim open. No money, starter-choice flags or original rewards change.
/// After either fossil is restored, the lab assistant or original bluff
/// researcher offers the other specimen for free, sharing a single claim flag.
/// Native item/gift failure leaves claims and specimens intact. Existing maps,
/// encounters, trainers, art, audio and other regions retain their content.
///
/// # Errors
/// Rejects duplicate installation, missing active starters/fossils, incompatible
/// late-access scripts, occupied labels or invalid runtime/geography output.
pub fn build_minnesota_research_access(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "early research already installed"
    );
    for species in ["POUCHID", "KINDLIT", "BIRCHIT", "TRILUME", "SIPHLET"] {
        ensure!(
            base.data.pokemon.contains_key(species)
                && base
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(&format!("MN_DEX_MEMBER_{species}")),
            "active native research species required: {species}"
        );
    }
    let mut pack = base.clone();
    starters(&mut pack)?;
    fossils(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    Ok(pack)
}

fn starters(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnWinonaFieldStation")?;
    let old = room
        .scripts
        .get("MnKindlitResearchAide")
        .context("installed research aide")?;
    let rows = old.as_array().context("native aide program")?;
    ensure!(
        rows.first() == Some(&c("scall", &["MN_W_AIDE"]))
            && rows.get(1) == Some(&c("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]))
            && rows
                .iter()
                .any(|row| row == &c("givepoke", &["KINDLIT", "5"])),
        "reviewed late research aide required"
    );
    for label in [
        "MnEarlyPartners",
        "MnEarlyPartnerWait",
        "MnEarlyPartnerFull",
        "MnEarlyPartnerClose",
    ] {
        ensure!(
            !room.scripts.contains_key(label),
            "occupied early partner label {label}"
        );
    }
    ensure!(
        pack.data
            .story_event_script_constants
            .global
            .contains_key("EVENT_MN_KINDLIT_RESEARCH_RECEIVED"),
        "prior Kindlit claim required"
    );
    for species in ["POUCHID", "BIRCHIT"] {
        flag(pack, &format!("EVENT_MN_{species}_RESEARCH_RECEIVED"))?;
    }
    put(
        &mut room,
        "MnKindlitResearchAide",
        vec![c("scall", &["MN_W_AIDE"]), c("sjump", &["MnEarlyPartners"])],
    );
    put(
        &mut room,
        "MnEarlyPartners",
        vec![
            c("checkevent", &[DELIVERED]),
            c("iffalse", &["MnEarlyPartnerWait"]),
            c("sjump", &["MnEarlyPartnerPOUCHID"]),
        ],
    );
    for (species, chosen, next) in [
        ("POUCHID", "CHIKORITA", "MnEarlyPartnerKINDLIT"),
        ("KINDLIT", "CYNDAQUIL", "MnEarlyPartnerBIRCHIT"),
        ("BIRCHIT", "TOTODILE", "MnEarlyPartnerDone"),
    ] {
        let root = format!("MnEarlyPartner{species}");
        let claim = format!("EVENT_MN_{species}_RESEARCH_RECEIVED");
        let initial = format!("EVENT_MN_WINONA_{chosen}");
        let offer = format!("{root}Text");
        let failed = format!("{root}Failed");
        for label in [&root, &offer, &failed] {
            ensure!(
                !room.scripts.contains_key(label),
                "occupied partner label {label}"
            );
        }
        put(
            &mut room,
            &root,
            vec![
                c("checkevent", &[&initial]),
                c("iftrue", &[next]),
                c("checkevent", &[&claim]),
                c("iftrue", &[next]),
                c("opentext", &[]),
                c("writetext", &[&offer]),
                c("yesorno", &[]),
                c("iffalse", &["MnEarlyPartnerClose"]),
                c("readvar", &["VAR_PARTYCOUNT"]),
                c("ifequal", &["6", "MnEarlyPartnerFull"]),
                c("closetext", &[]),
                c("givepoke", &[species, "5"]),
                c("ifequal", &["2", &failed]),
                c("setevent", &[&claim]),
                c("opentext", &[]),
                c("writetext", &["MnEarlyPartnerThanksText"]),
                c("waitbutton", &[]),
                c("sjump", &["MnEarlyPartnerClose"]),
            ],
        );
        put(
            &mut room,
            &failed,
            vec![c("opentext", &[]), c("sjump", &["MnEarlyPartnerFull"])],
        );
        text(
            &mut room,
            &offer,
            &[
                "Your first parcel",
                "helped our field",
                "research. This",
                &format!("{species} is free."),
                "Take it with you?",
            ],
        )?;
    }
    for (entry, label, lines) in [
        (
            "MnEarlyPartnerWait",
            "MnEarlyPartnerWaitText",
            vec![
                "Deliver TAMARACK's",
                "parcel to the",
                "ROCHESTER clinic.",
                "Then return here",
                "for free starter",
                "partners.",
            ],
        ),
        (
            "MnEarlyPartnerDone",
            "MnEarlyPartnerDoneText",
            vec![
                "All three starter",
                "lines can travel",
                "with you now.",
                "Keep researching!",
            ],
        ),
    ] {
        put(&mut room, entry, vec![c("jumptext", &[label])]);
        text(&mut room, label, &lines)?;
    }
    put(
        &mut room,
        "MnEarlyPartnerFull",
        vec![
            c("writetext", &["MnEarlyPartnerFullText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnEarlyPartnerClose"]),
        ],
    );
    put(
        &mut room,
        "MnEarlyPartnerClose",
        vec![c("closetext", &[]), c("end", &[])],
    );
    text(
        &mut room,
        "MnEarlyPartnerFullText",
        &[
            "Your party is full",
            "Leave one space",
            "and return. I'll",
            "keep it for you.",
        ],
    )?;
    text(
        &mut room,
        "MnEarlyPartnerThanksText",
        &[
            "A new field friend",
            "Ask me again for",
            "the other starter",
            "you did not pick.",
        ],
    )?;
    text(
        &mut room,
        "MnWTamarackDeliveryReturnText",
        &[
            "ROCHESTER called.",
            "All pages arrived.",
            "Ask our aide for",
            "the other two free",
            "starter partners.",
        ],
    )?;
    publish(pack, room)
}

fn fossils(pack: &mut CompiledGamePack) -> Result<()> {
    let mut bluff = map(pack, "MnR01C00R00")?;
    let rows = bluff
        .scripts
        .get("MnFossilExchange")
        .and_then(|v| v.as_array())
        .context("installed fossil exchange")?;
    ensure!(
        rows.get(2) == Some(&c("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]))
            && rows.get(3) == Some(&c("iffalse", &["MnFossilBluffRepeat"])),
        "reviewed late fossil exchange required"
    );
    let mut revised = rows.clone();
    revised.splice(
        2..4,
        [
            c("checkevent", &["EVENT_MN_TRILUME_RESTORED"]),
            c("iftrue", &["MnFossilEarlyExchangeOffer"]),
            c("checkevent", &["EVENT_MN_SIPHLET_RESTORED"]),
            c("iffalse", &["MnFossilBluffRepeat"]),
            c("sjump", &["MnFossilEarlyExchangeOffer"]),
        ],
    );
    let offer = revised.split_off(7);
    ensure!(
        !bluff.scripts.contains_key("MnFossilEarlyExchangeOffer"),
        "occupied fossil offer label"
    );
    put(&mut bluff, "MnFossilExchange", revised);
    put(&mut bluff, "MnFossilEarlyExchangeOffer", offer);
    text(
        &mut bluff,
        "MnFossilExchangeText",
        &[
            "Your first fossil",
            "is alive! Take the",
            "other specimen for",
            "free research?",
        ],
    )?;
    text(
        &mut bluff,
        "MnFossilDirectionsText",
        &[
            "FOSSIL LAB beside",
            "ROCHESTER clinic.",
            "Earn the CARE",
            "BADGE first.",
            "Leave a party slot",
            "Restore one. Then",
            "ask the assistant",
            "for the other free",
            "fossil specimen.",
        ],
    )?;
    publish(pack, bluff)?;

    let mut lab = map(pack, "MnRochesterFossilLab")?;
    ensure!(
        lab.scripts.get("MnRochesterFossilLabAssistant")
            == Some(&serde_json::json!([c(
                "jumptextfaceplayer",
                &["MnRochesterFossilLabAssistantText"]
            )])),
        "reviewed fossil assistant required"
    );
    for name in [
        "MnLabSecondOffer",
        "MnLabSecondLens",
        "MnLabSecondResult",
        "MnLabSecondFull",
        "MnLabSecondClose",
        "MnLabSecondDirections",
    ] {
        ensure!(
            !lab.scripts.contains_key(name),
            "occupied second fossil label {name}"
        );
    }
    put(
        &mut lab,
        "MnRochesterFossilLabAssistant",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[EXCHANGED]),
            c("iftrue", &["MnLabSecondDirections"]),
            c("checkevent", &["EVENT_MN_TRILUME_RESTORED"]),
            c("iftrue", &["MnLabSecondOffer"]),
            c("checkevent", &["EVENT_MN_SIPHLET_RESTORED"]),
            c("iftrue", &["MnLabSecondOffer"]),
            c("jumptext", &["MnRochesterFossilLabAssistantText"]),
        ],
    );
    put(
        &mut lab,
        "MnLabSecondOffer",
        vec![
            c("opentext", &[]),
            c("writetext", &["MnLabSecondOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnLabSecondClose"]),
            c("checkevent", &["EVENT_MN_FOSSIL_CHOSE_LENS"]),
            c("iffalse", &["MnLabSecondLens"]),
            c("verbosegiveitem", &["SPIRE_FOSSIL", "1"]),
            c("sjump", &["MnLabSecondResult"]),
        ],
    );
    put(
        &mut lab,
        "MnLabSecondLens",
        vec![
            c("verbosegiveitem", &["LENS_FOSSIL", "1"]),
            c("sjump", &["MnLabSecondResult"]),
        ],
    );
    put(
        &mut lab,
        "MnLabSecondResult",
        vec![
            c("iffalse", &["MnLabSecondFull"]),
            c("setevent", &[EXCHANGED]),
            c("writetext", &["MnLabSecondThanksText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnLabSecondClose"]),
        ],
    );
    put(
        &mut lab,
        "MnLabSecondFull",
        vec![
            c("writetext", &["MnLabSecondFullText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnLabSecondClose"]),
        ],
    );
    put(
        &mut lab,
        "MnLabSecondClose",
        vec![c("closetext", &[]), c("end", &[])],
    );
    put(
        &mut lab,
        "MnLabSecondDirections",
        vec![c("jumptext", &["MnLabSecondThanksText"])],
    );
    text(
        &mut lab,
        "MnRochesterFossilLabAssistantText",
        &[
            "Pick a free LENS",
            "or SPIRE FOSSIL on",
            "DRIFTLESS TRAIL.",
            "Earn the CARE",
            "BADGE first.",
            "restore it here.",
            "Then I'll give you",
            "the other fossil",
            "at no charge.",
        ],
    )?;
    text(
        &mut lab,
        "MnLabSecondOfferText",
        &[
            "Your first fossil",
            "is alive! We have",
            "the other specimen",
            "ready for you.",
            "Take it for free?",
        ],
    )?;
    text(
        &mut lab,
        "MnLabSecondThanksText",
        &[
            "Our scientist can",
            "restore both lines",
            "Leave a party slot",
            "for each friend.",
            "TRILUME evolves",
            "at level 34, as",
            "does SIPHLET.",
            "CASTOROCK lives in",
            "the Route 3 cave.",
            "Bring STRENGTH and",
            "read the jaw clues",
        ],
    )?;
    text(
        &mut lab,
        "MnLabSecondFullText",
        &[
            "Your KEY ITEMS",
            "pocket is full.",
            "The fossil will",
            "wait here for you.",
        ],
    )?;
    text(
        &mut lab,
        "MnLabFossilRestoredText",
        &[
            "An ancient friend!",
            "Ask our assistant",
            "for the other free",
            "fossil specimen.",
        ],
    )?;
    publish(pack, lab)
}

fn text(room: &mut MapModule, label: &str, lines: &[&str]) -> Result<()> {
    ensure!(
        lines.iter().all(|line| line.chars().count() <= 18),
        "research text exceeds native width: {label}"
    );
    authoring::text(room, label, lines);
    Ok(())
}

#[cfg(test)]
#[path = "minnesota_research_access_tests.rs"]
mod tests;
