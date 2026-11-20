//! Free early research partners and both restored fossil families in one run.
use anyhow::{Context, Result, ensure};

use crate::authored_dialogue;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring::flag;
use crate::northstar_rival_arc::{map, publish, put};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-early-research-access-v1";
const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";
const EXCHANGED: &str = "EVENT_MN_FOSSIL_EXCHANGED";

/// Syncs the three Field Station balls with the original gift and later adoptions.
pub(crate) fn refresh_adopted_starter_benches(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = map(pack, "MnWinonaFieldStation")?;
    const SYNC: &str = "MnWinonaResearchBenches";
    ensure!(
        !room.scripts.contains_key(SYNC),
        "starter benches already revised"
    );
    let mut sync = Vec::new();
    for (species, original) in [
        ("POUCHID", "CHIKORITA"),
        ("KINDLIT", "CYNDAQUIL"),
        ("BIRCHIT", "TOTODILE"),
    ] {
        let actor = format!("MN_W_{original}");
        let picked = format!("EVENT_MN_WINONA_{original}");
        let adopted = format!("EVENT_MN_{species}_RESEARCH_RECEIVED");
        let ball = room
            .objects
            .iter_mut()
            .find(|obj| obj.object_identifier.as_deref() == Some(actor.as_str()))
            .with_context(|| format!("missing Winona starter ball for {species}"))?;
        ensure!(
            ball.event_flag == picked,
            "expected original starter visibility for {species}"
        );
        // 'disappear' writes the object's event flag. An adoption must never count as
        // the player's original starter choice. Reconstruct temporary visibility on entry.
        ball.event_flag = "-1".into();
        let program = room
            .scripts
            .get_mut(&format!("MnEarlyPartner{species}"))
            .and_then(serde_json::Value::as_array_mut)
            .context("research adoption program")?;
        let success = program
            .iter()
            .position(|row| row == &c("setevent", &[&adopted]))
            .context("successful starter adoption")?;
        program.insert(success + 1, c("disappear", &[&actor]));

        let check = format!("MnWinonaBench{species}");
        let hide = format!("{check}Hide");
        sync.push(c("scall", &[&check]));
        put(
            &mut room,
            &check,
            vec![
                c("checkevent", &[&picked]),
                c("iftrue", &[&hide]),
                c("checkevent", &[&adopted]),
                c("iftrue", &[&hide]),
                c("end", &[]),
            ],
        );
        put(
            &mut room,
            &hide,
            vec![c("disappear", &[&actor]), c("end", &[])],
        );
    }
    sync.push(c("endcallback", &[]));
    put(&mut room, SYNC, sync);
    crate::northstar_lair_authoring::callback(&mut room, "MAPCALLBACK_OBJECTS", SYNC);
    publish(pack, room)
}

/// Rewrites the existing Rochester lab conversations on a completed pack.
///
/// # Errors
/// Rejects invalid packs, a missing lab or dialogue, and invalid revised content.
pub fn revise_minnesota_fossil_lab_dialogue(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|entry| entry == "minnesota-fossil-lab-dialogue-v1"),
        "fossil lab dialogue already revised"
    );
    let mut pack = base.clone();
    let mut lab = map(&pack, "MnRochesterFossilLab")?;
    for label in [
        "MnRochesterFossilLabAssistantText",
        "MnLabFossilNeedSpecimenText",
        "MnLabSecondThanksText",
    ] {
        ensure!(lab.scripts.contains_key(label), "missing lab text {label}");
        authored_dialogue::install_canonical(&mut lab, label)?;
    }
    publish(&mut pack, lab)?;
    pack.report
        .manifests
        .push("minnesota-fossil-lab-dialogue-v1".into());
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
    crate::northstar_reward_receipts::install(&mut pack)?;
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
    for label in ["MnFossilExchangeText", "MnFossilDirectionsText"] {
        authored_dialogue::install_canonical(&mut bluff, label)?;
    }
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
    for label in [
        "MnRochesterFossilLabAssistantText",
        "MnLabFossilNeedSpecimenText",
        "MnLabSecondOfferText",
        "MnLabSecondThanksText",
        "MnLabSecondFullText",
        "MnLabFossilRestoredText",
    ] {
        authored_dialogue::install_canonical(&mut lab, label)?;
    }
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
