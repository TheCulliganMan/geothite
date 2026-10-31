//! Repair the precise receipt boundary emitted by the older Minnesota reward.
use anyhow::{Context, Result};

use super::BevyRuntimeShell;

/// The older verbose Master Ball grant saved its previous introduction wait
/// after putting the ball in the bag. Continue replayed the gift. Recognize only
/// that successful, unclaimed handoff and advance it to the saved confirmation.
/// Full-bag refusal and a merely offered reward do not match this signature.
pub(super) fn restore_legacy_receipt(shell: &mut BevyRuntimeShell) -> Result<()> {
    let data = shell.shell.runtime().data();
    let Some(room) = data.maps.get("MnWinonaFieldStation") else {
        return Ok(());
    };
    let Some(program) = room
        .scripts
        .get("MnRewardTamarack")
        .and_then(serde_json::Value::as_array)
    else {
        return Ok(());
    };
    // Only the repaired pack can migrate the old handoff; keep other packs exact.
    if !program
        .get(7)
        .is_some_and(|r| r["command"] == "giveitem" && r["args"][0] == "MASTER_BALL")
    {
        return Ok(());
    }
    let master = data
        .items
        .get("MASTER_BALL")
        .context("native Master Ball")?;
    let state = shell.shell.session().state();
    if shell.shell.session().overworld().map.name != "MnWinonaFieldStation"
        || !state.flags.is_event_flag_set("EVENT_MN_SURVEY_RESTORED")?
        || state
            .flags
            .is_event_flag_set("EVENT_MN_MASTER_BALL_RECEIVED")?
        || state.bag.quantity(master) == 0
        || state.script_runtime.script_value.as_deref() != Some("1")
        || state.script_runtime.named_buffers.get("STRING_BUFFER_1") != Some(&master.name)
        || state.script_runtime.pending_text_label.as_deref() != Some("MnRewardMasterIntroText")
        || !state
            .script_runtime
            .pending_text_wait
            .as_ref()
            .is_some_and(|wait| wait.source_script == "MnRewardTamarack" && wait.command_index == 6)
    {
        return Ok(());
    }
    let state = shell.shell.session_mut().state_mut();
    state
        .flags
        .set_event_flag("EVENT_MN_MASTER_BALL_RECEIVED", true)?;
    state.script_runtime.pending_text_label = Some("MnRewardMasterGivenText".into());
    state
        .script_runtime
        .pending_text_wait
        .as_mut()
        .context("matched old introduction wait")?
        .command_index = 11;
    // A saved later introduction page belongs to the old text owner.
    state
        .script_runtime
        .memory
        .remove("GEOTHITE_FIELD_DIALOGUE_PAGE_V1");
    Ok(())
}
