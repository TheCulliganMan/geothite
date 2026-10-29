//! Rebuild the frontend cursor from an authoritative saved text boundary.
use anyhow::{Context, Result};

use super::{BevyRuntimeShell, arm_visible_active_script_cursor};
use crate::core::state::ScriptTextRuntimeKind;

pub(super) fn restore(shell: &mut BevyRuntimeShell) -> Result<()> {
    let memory = &shell.shell.session().state().script_runtime;
    let continuation = if let Some(wait) = &memory.pending_text_wait {
        Some((wait.source_script.clone(), wait.command_index))
    } else if let Some(choice) = &memory.pending_yes_no {
        Some((choice.source_script.clone(), choice.command_index))
    } else if let Some(label) = &memory.pending_text_label {
        // PrintText can be saved before its following promptbutton executes.
        // The live label alone does not identify that call site; its native
        // Write event does, including when a subroutine owns the conversation.
        memory.text_events.iter().rev().find_map(|event| {
            (event.kind == ScriptTextRuntimeKind::Write && event.text_label.as_ref() == Some(label))
                .then(|| (event.source_script.clone(), event.command_index))
        })
    } else {
        None
    };
    if let Some((script, index)) = continuation {
        let next = index
            .checked_add(1)
            .context("saved dialogue continuation index overflow")?;
        arm_visible_active_script_cursor(shell, &script, next);
    }
    Ok(())
}
