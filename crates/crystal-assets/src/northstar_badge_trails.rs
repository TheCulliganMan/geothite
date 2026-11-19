//! Directions through Driftless Hollow and the badge journey.
use anyhow::{Context, Result, ensure};

use serde_json::{Value, json};

use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

/// Revises the Driftless exits, field guide, and directions from three gym leaders.
///
/// # Errors
///
/// Returns an error for an unexpected parent or map program, or if the
/// revised pack does not pass the runtime verifier.
pub fn revise_badge_trails(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-shore-watch-v1"),
        "Badge Trails requires a Shore Watch edition"
    );
    let mut pack = base.clone();
    crate::northstar_badge_trails_hollow::apply(&mut pack)?;
    crate::northstar_badge_trails_leaders::apply(&mut pack)?;
    for (map, label) in [
        ("MnStPaulGym", "MnStPaulGymRequirementsText"),
        ("MnDuluthGym", "MnDuluthGymRequirementsText"),
    ] {
        let mut room = pack
            .data
            .maps
            .get(map)
            .context("badge prerequisite room")?
            .clone();
        let original = room.scripts.get(label).context("badge prerequisite text")?;
        ensure!(
            pack.data.map_scripts.get(label) == Some(original)
                && room.script_text_bodies.contains_key(label),
            "missing or inconsistent prerequisite text in {map}"
        );
        if map == "MnStPaulGym" {
            separate_st_paul_court_admission(&mut room)?;
        }
        authored_dialogue::install_canonical(&mut room, label)?;
        room.script_text_bodies = crate::parse_script_text_bodies(map, &room.scripts)?;
        authored_town_publication::replace(&mut pack, room)?;
    }
    pack.report
        .manifests
        .push("northstar-badge-trails-v1".into());
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

// Someone with the Current Badge only needs a win in one wing. This path
// also handles a saved challenger already standing beyond the grilles.
fn separate_st_paul_court_admission(room: &mut MapModule) -> Result<()> {
    let court = "MnStPaulGymCourtRequirements";
    let leader = room
        .scripts
        .get_mut("MnStPaulGymLeader")
        .and_then(Value::as_array_mut)
        .context("Brannic's challenge")?;
    let check = command("checkevent", &["EVENT_MN_GYM_4_COURT_OPEN"]);
    let denial = command("iffalse", &["MnStPaulGymRequirements"]);
    let index = leader
        .windows(2)
        .position(|pair| pair[0] == check && pair[1] == denial)
        .context("St Paul court check")?;
    leader[index + 1] = command("iffalse", &[court]);
    ensure!(
        room.scripts
            .insert(
                court.into(),
                json!([command("jumptext", &["MnStPaulGymGuideText"])])
            )
            .is_none(),
        "St Paul court directions already installed"
    );
    room.script_control_commands = crate::parse_script_control_commands(&room.id, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(&room.id, &room.scripts)?;
    Ok(())
}
