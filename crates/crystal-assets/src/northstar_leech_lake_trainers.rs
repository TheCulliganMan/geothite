//! The working trail crew, casting bank and rink regular on the Leech Lake approach.

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::northstar_battle_balance::{AuthoredMember, build_member};
use crate::northstar_regional_roster::RegionalRoster;

const PARTIES: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/leech-lake-trainers-v01/parties.json");
const PARENT_HASHES: [&str; 2] = [
    "ee51847328287a66b358f951e0c49db99ae8c35dd2925441a73ac290b2ccdfb5",
    "db259dcaabc1753f3dffc0df7855e06a41ee8fa2d04498bfd081815529e3eaae",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Team {
    trainer: String,
    slots: Vec<Slot>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Slot {
    index: usize,
    original: String,
    new_member: AuthoredMember,
}

/// Revises three Route 7 trainers for the Trail Signs editions.
///
/// # Errors
/// Returns an error for an unexpected parent or party, an illegal regional member
/// or moveset, or a resulting archive that fails runtime verification.
pub fn revise_leech_lake_trainers(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-trail-signs-v1")
            && PARENT_HASHES.contains(&pack.identity.content_hash.as_str()),
        "Leech Lake trainers require a matching Trail Signs archive"
    );
    let teams: [Team; 3] = serde_json::from_slice(PARTIES)?;
    let roster = RegionalRoster::current()?;
    for team in teams {
        for slot in team.slots {
            let original = pack
                .data
                .trainers
                .get(&team.trainer)
                .and_then(|trainer| trainer.party.get(slot.index))
                .with_context(|| format!("missing {} party slot {}", team.trainer, slot.index))?;
            ensure!(
                original.species == slot.original
                    && original.level == slot.new_member.level
                    && original.item == slot.new_member.item,
                "unexpected {} party slot {}",
                team.trainer,
                slot.index
            );
            let replacement = build_member(&pack, &roster, &slot.new_member, original.dvs)?;
            pack.data
                .trainers
                .trainers
                .get_mut(&team.trainer)
                .context("Route 7 trainer retained")?
                .party[slot.index] = replacement;
        }
    }
    pack.report
        .manifests
        .push("northstar-leech-lake-trainers-v1".into());
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
