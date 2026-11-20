//! Late trainer move choices and an optional Brainerd Gym guide hint.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_battle_balance::{AuthoredMember, build_member};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, authored_town_publication};

#[cfg(test)]
#[path = "badge_battle_moves.rs"]
mod badge_battle_moves;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parties {
    parties: Vec<TrainerEdits>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrainerEdits {
    trainer_id: String,
    move_edits: Vec<MoveEdit>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveEdit {
    species: String,
    level: u8,
    slot: usize,
    before: String,
    after: String,
}

fn trainer_moves(pack: &mut CompiledGamePack) -> Result<()> {
    let recipe: Parties = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/badge-battles-v01/league.json"
    ))?;
    let policy = RegionalRoster::current()?;
    for trainer in recipe.parties {
        for edit in trainer.move_edits {
            let party = &pack
                .data
                .trainers
                .get(&trainer.trainer_id)
                .context("installed trainer")?
                .party;
            let index = party
                .iter()
                .position(|m| m.species == edit.species && m.level == edit.level)
                .with_context(|| {
                    format!(
                        "{} has no level {} {}",
                        trainer.trainer_id, edit.level, edit.species
                    )
                })?;
            let old = &party[index];
            let mut moves: Vec<_> = old.moves.iter().map(|m| m.name.clone()).collect();
            ensure!(
                moves.get(edit.slot) == Some(&edit.before),
                "unexpected move on {}/{} at slot {}",
                trainer.trainer_id,
                edit.species,
                edit.slot
            );
            moves[edit.slot] = edit.after;
            let updated = build_member(
                pack,
                &policy,
                &AuthoredMember {
                    species: old.species.clone(),
                    level: old.level,
                    moves,
                    item: old.item.clone(),
                },
                old.dvs,
            )?;
            pack.data
                .trainers
                .trainers
                .get_mut(&trainer.trainer_id)
                .context("retained trainer")?
                .party[index] = updated;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct Hint {
    map: String,
    label: String,
    before: Option<Vec<AuthoredDialoguePage>>,
    after: Vec<AuthoredDialoguePage>,
}

fn lake_guide(pack: &mut CompiledGamePack) -> Result<()> {
    let hints: Vec<Hint> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/badge-battles-v01/lake-guide.json"
    ))?;
    const GYM: &str = "MnBrainerdGym";
    const GUIDE: &str = "MnBrainerdGymCircuitBeforeChampion";
    const BADGE: &str = "MnBadgeBattleGuideBadge";
    const REPORT: &str = "MnBadgeBattleGuideReport";
    let mut room = pack.data.maps.get(GYM).context(GYM)?.clone();
    let mut texts = BTreeMap::new();
    for hint in hints {
        ensure!(hint.map == GYM, "unexpected guide map");
        let before = hint
            .before
            .as_deref()
            .map(authored_dialogue::compile)
            .transpose()?;
        ensure!(
            room.scripts.get(&hint.label) == before.as_ref()
                && pack.data.map_scripts.get(&hint.label) == before.as_ref(),
            "unexpected prior text {}",
            hint.label
        );
        authored_dialogue::install_canonical(&mut room, &hint.label)?;
        let after = authored_dialogue::compile(&hint.after)?;
        ensure!(
            room.scripts.get(&hint.label) == Some(&after),
            "guide text differs from authored revision"
        );
        texts.insert(hint.label, after);
    }
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(GYM, &texts)?);
    let original = json!([command("jumptextfaceplayer", &["MnBrainerdGymGuideText"])]);
    ensure!(
        room.scripts.get(GUIDE) == Some(&original)
            && pack.data.map_scripts.get(GUIDE) == Some(&original),
        "unexpected pre-Champion gym guide"
    );
    for label in [BADGE, REPORT] {
        ensure!(
            !pack.data.map_scripts.contains_key(label) && !room.scripts.contains_key(label),
            "guide label {label} is already in use"
        );
    }
    room.scripts.insert(
        GUIDE.into(),
        json!([
            command("checkevent", &["EVENT_MN_GYM_6_DEFEATED"]),
            command("iftrue", &[BADGE]),
            command("jumptextfaceplayer", &["MnBrainerdGymGuideText"]),
        ]),
    );
    room.scripts.insert(
        BADGE.into(),
        json!([
            command("checkevent", &["EVENT_MN_BRAINERD_REPORT"]),
            command("iftrue", &[REPORT]),
            command("jumptextfaceplayer", &["MnBrainerdGymGuideAfterBadgeText"]),
        ]),
    );
    room.scripts.insert(
        REPORT.into(),
        json!([command(
            "jumptextfaceplayer",
            &["MnBrainerdGymGuideAfterReportText"]
        )]),
    );
    room.script_flag_commands = crate::parse_script_flag_commands(GYM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(GYM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(GYM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}

/// Revises the late trainer moves and Brainerd's pre-badge guidance.
///
/// # Errors
/// Returns an error if the Fen Waterline parent or the expected party and
/// dialogue records differ, a move is illegal, or the archive fails validation.
pub fn revise_badge_battles(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-fen-waterline-v1"),
        "Badge Battles requires a Fen Waterline edition"
    );
    trainer_moves(&mut pack)?;
    lake_guide(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-badge-battles-v1".into());
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
