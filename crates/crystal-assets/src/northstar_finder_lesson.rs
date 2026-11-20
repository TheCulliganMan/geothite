//! Winona's Itemfinder lesson and the research aide's full-pocket response.
#[path = "northstar_winona_riverbank.rs"]
mod northstar_winona_riverbank;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const STATION: &str = "MnWinonaFieldStation";
const BANK: &str = "MnRewardFinderBank";
const FULL: &str = "MnRewardAideFull";

/// Gives a newly equipped Minnesota researcher a nearby, unclaimed practice site.
///
/// # Errors
/// Rejects an unexpected pack or changed reward program, occupied script labels,
/// invalid native dialogue, or an inconsistent compiled archive.
pub fn revise_finder_lesson(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-ely-challenge-v1"),
        "Finder Lesson requires the Ely Challenge edition"
    );
    let mut station = pack
        .data
        .maps
        .get(STATION)
        .context("Winona Field Station")?
        .clone();
    for label in [BANK, FULL, "MnRewardFinderBankText", "MnRewardAideFullText"] {
        ensure!(
            !pack.data.map_scripts.contains_key(label) && !station.scripts.contains_key(label),
            "occupied {label}"
        );
    }
    for tier in 0..5 {
        let label = format!("MnRewardDexClaim{tier}");
        let program = station
            .scripts
            .get_mut(&label)
            .and_then(|p| p.as_array_mut())
            .context("research award program")?;
        ensure!(
            program.len() == 12
                && program[4] == command("iffalse", &["MnRewardStationFull"])
                && program[5] == command("setevent", &[&format!("EVENT_MN_DEX_REWARD_{tier}")])
                && program[10] == command("closetext", &[])
                && program[11] == command("end", &[]),
            "unexpected {label}"
        );
        program[4] = command("iffalse", &[FULL]);
        if tier == 1 {
            ensure!(
                program[3] == command("giveitem", &["ITEMFINDER", "1"])
                    && program[8] == command("writetext", &["MnRewardDexClaim1ClaimReceipt3Text"]),
                "Itemfinder handoff"
            );
            program.splice(
                10..10,
                [
                    command("checkevent", &["EVENT_MN_HIDDEN_WINONA"]),
                    command("iffalse", &[BANK]),
                ],
            );
        }
    }
    for (script, text) in [
        (BANK, "MnRewardFinderBankText"),
        (FULL, "MnRewardAideFullText"),
    ] {
        put(
            &mut station,
            script,
            vec![
                command("writetext", &[text]),
                command("waitbutton", &[]),
                command("closetext", &[]),
                command("end", &[]),
            ],
        );
    }
    ensure!(
        pack.data.maps["MnWinona"].scripts["MN_HIDDEN_WINONA"]
            == json!([command(
                "hiddenitem",
                &["POKE_BALL", "EVENT_MN_HIDDEN_WINONA"]
            )]),
        "practice ball or flag changed"
    );
    for label in [
        "MnRewardDexClaim1Text",
        "MnRewardFinderBankText",
        "MnRewardAideFullText",
    ] {
        authored_dialogue::install_canonical(&mut station, label)?;
    }
    station.script_control_commands =
        crate::parse_script_control_commands(STATION, &station.scripts)?;
    station.script_flag_commands = crate::parse_script_flag_commands(STATION, &station.scripts)?;
    station.script_text_commands = crate::parse_script_text_commands(STATION, &station.scripts)?;
    station.script_text_bodies = crate::parse_script_text_bodies(STATION, &station.scripts)?;
    authored_town_publication::replace(&mut pack, station)?;
    northstar_winona_riverbank::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-finder-lesson-v1".into());
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
