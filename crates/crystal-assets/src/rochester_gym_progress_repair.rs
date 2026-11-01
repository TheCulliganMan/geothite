//! Repair Rochester's practice-battle guidance and leader qualification.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, northstar_rival_arc};

const MANIFEST: &str = "minnesota-rochester-gym-progress-repair-v1";

/// Repairs the existing Gym without changing earned flags, parties, or terrain.
///
/// # Errors
/// Rejects an incompatible Gym or invalid compiled output.
pub fn repair_rochester_gym_progress(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "repair already installed"
    );
    let mut pack = base.clone();
    let mut room = northstar_rival_arc::map(&pack, "MnRochesterGym")?;
    let trainer = room
        .scripts
        .get_mut("MN_GYM_1_TRAINER_2")
        .context("Elvik script")?
        .as_array_mut()
        .context("Elvik commands")?;
    ensure!(
        trainer.get(3) == Some(&c("checkevent", &["EVENT_MN_GYM_1_TRAINER_1_DEFEATED"]))
            && trainer.get(4) == Some(&c("iffalse", &["MnRochesterGymRequirements"])),
        "unexpected Elvik prerequisite"
    );
    trainer[4] = c("iffalse", &["MnRochesterGymBeatTavel"]);
    let leader = room
        .scripts
        .get_mut("MnRochesterGymLeader")
        .context("leader script")?
        .as_array_mut()
        .context("leader commands")?;
    ensure!(
        leader.get(5) == Some(&c("checkevent", &["EVENT_MN_GYM_1_COURT_OPEN"]))
            && leader.get(6) == Some(&c("iffalse", &["MnRochesterGymCourtRequirements"])),
        "unexpected leader prerequisite"
    );
    leader.splice(
        5..7,
        [
            c("checkevent", &["EVENT_MN_GYM_1_TRAINER_1_DEFEATED"]),
            c("iffalse", &["MnRochesterGymBeatTavel"]),
            c("checkevent", &["EVENT_MN_GYM_1_TRAINER_2_DEFEATED"]),
            c("iffalse", &["MnRochesterGymBeatElvik"]),
        ],
    );
    for (label, lines) in [
        (
            "MnRochesterGymBeatTavel",
            ["Beat TAVEL first.", "He's by the door."],
        ),
        (
            "MnRochesterGymBeatElvik",
            ["Beat ELVIK next.", "He's up ahead."],
        ),
    ] {
        let text = format!("{label}Text");
        authoring::text(&mut room, &text, &lines);
        room.scripts
            .insert(label.into(), json!([c("jumptext", &[&text])]));
    }
    northstar_rival_arc::publish(&mut pack, room)?;
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
