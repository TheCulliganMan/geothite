//! Retain evolution acceptance until its move choices and Dex commit finish.
use anyhow::Result;

use super::{
    BevyRuntimeShell, record_visible_completed_evolution, visible_evolution_moves_resolved,
};

/// Record only accepted, fully resolved evolutions before battle text exits.
/// Unresolved entries remain available to the move-result completion callback.
/// Returns an error if a completed recipient no longer exists in the party.
pub(super) fn record_completed_before_exit(shell: &mut BevyRuntimeShell) -> Result<()> {
    let pending = shell
        .battle_evolution_cancellations
        .iter()
        .map(|entry| {
            (
                entry.party_index,
                entry.accepted,
                visible_evolution_moves_resolved(shell, entry),
            )
        })
        .collect::<Vec<_>>();
    let mut completed = Vec::new();
    for (party_index, accepted, moves_resolved) in pending {
        #[cfg(test)]
        if std::env::var_os("CRYSTAL_EVOLUTION_TRACE").is_some() {
            shell.last_audio_events.push(format!(
                "evolution exit party={party_index} accepted={accepted} moves_resolved={moves_resolved}"
            ));
            super::trim_event_log(&mut shell.last_audio_events);
        }
        if accepted && moves_resolved {
            record_visible_completed_evolution(shell, party_index)?;
            completed.push(party_index);
        }
    }
    shell
        .battle_evolution_cancellations
        .retain(|entry| !completed.contains(&entry.party_index));
    Ok(())
}
