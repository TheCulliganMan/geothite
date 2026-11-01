//! Close the one-run Fire-starter acquisition gap through the existing aide.
use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring::flag;
use crate::northstar_rival_arc::{map, publish, put};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-kindlit-research-v1";
const CLAIMED: &str = "EVENT_MN_KINDLIT_RESEARCH_RECEIVED";
const ENTRY: &str = "MnKindlitResearchAide";

/// Adds a once-only level-five Kindlit research partner after the regional
/// Champion. The existing Winona aide still completes the original research
/// conversation first. Refusal or a full party leaves the partner available;
/// only successful native delivery commits the claim. Starter-choice state,
/// existing research programs, terrain, actors, encounters and audio remain.
///
/// # Errors
/// Rejects duplicate installation, incompatible geography, inactive Kindlit,
/// missing Champion/aide identities, occupied script labels or invalid output.
pub fn build_minnesota_starter_return(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "research partner already installed"
    );
    for constant in ["MN_DEX_MEMBER_KINDLIT", "EVENT_MN_LEAGUE_CHAMPION"] {
        ensure!(
            base.data
                .story_event_script_constants
                .global
                .contains_key(constant),
            "missing native research prerequisite {constant}"
        );
    }
    ensure!(
        base.data.pokemon.contains_key("KINDLIT"),
        "native Fire starter required"
    );
    let mut pack = base.clone();
    flag(&mut pack, CLAIMED)?;
    let mut room = map(&pack, "MnWinonaFieldStation")?;
    let aide = room
        .objects
        .iter_mut()
        .find(|object| object.object_identifier.as_deref() == Some("MN_W_AIDE"))
        .context("existing Winona research aide")?;
    let original = aide.script.clone();
    ensure!(
        room.scripts.contains_key(&original),
        "existing aide program required"
    );
    aide.script = ENTRY.into();
    for label in [
        ENTRY,
        "MnKindlitFull",
        "MnKindlitGiftFailed",
        "MnKindlitClose",
        "MnKindlitEnd",
        "MnKindlitOfferText",
        "MnKindlitFullText",
        "MnKindlitThanksText",
    ] {
        ensure!(
            !room.scripts.contains_key(label),
            "occupied research script {label}"
        );
    }
    put(
        &mut room,
        ENTRY,
        vec![
            c("scall", &[&original]),
            c("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]),
            c("iffalse", &["MnKindlitEnd"]),
            c("checkevent", &[CLAIMED]),
            c("iftrue", &["MnKindlitEnd"]),
            c("opentext", &[]),
            c("writetext", &["MnKindlitOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnKindlitClose"]),
            c("readvar", &["VAR_PARTYCOUNT"]),
            c("ifequal", &["6", "MnKindlitFull"]),
            c("closetext", &[]),
            c("givepoke", &["KINDLIT", "5"]),
            c("ifequal", &["2", "MnKindlitGiftFailed"]),
            c("setevent", &[CLAIMED]),
            c("opentext", &[]),
            c("writetext", &["MnKindlitThanksText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnKindlitClose"]),
        ],
    );
    put(
        &mut room,
        "MnKindlitGiftFailed",
        vec![c("opentext", &[]), c("sjump", &["MnKindlitFull"])],
    );
    put(
        &mut room,
        "MnKindlitFull",
        vec![
            c("writetext", &["MnKindlitFullText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnKindlitClose"]),
        ],
    );
    put(
        &mut room,
        "MnKindlitClose",
        vec![c("closetext", &[]), c("end", &[])],
    );
    put(&mut room, "MnKindlitEnd", vec![c("end", &[])]);
    text(
        &mut room,
        "MnKindlitOfferText",
        &[
            "With your League",
            "work complete, our",
            "KINDLIT can join",
            "your field team.",
            "Take it with you?",
        ],
    )?;
    text(
        &mut room,
        "MnKindlitFullText",
        &[
            "Your party is full",
            "Make room for the",
            "new field partner.",
        ],
    )?;
    text(
        &mut room,
        "MnKindlitThanksText",
        &[
            "Record what you",
            "learn out there.",
            "TAMARACK will want",
            "to hear about it.",
        ],
    )?;
    publish(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
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

fn text(room: &mut MapModule, label: &str, lines: &[&str]) -> Result<()> {
    ensure!(
        lines.iter().all(|line| line.chars().count() <= 18),
        "research text exceeds native width"
    );
    crate::grand_marais_modpack::authoring::text(room, label, lines);
    Ok(())
}
