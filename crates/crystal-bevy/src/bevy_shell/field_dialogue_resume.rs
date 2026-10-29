//! Rebuild the frontend cursor from an authoritative saved text boundary.
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use super::{
    BevyRuntimeShell, VisibleFieldTextReveal, arm_visible_active_script_cursor,
    visible_field_dialog_pages,
};
use crate::core::state::ScriptTextRuntimeKind;

const PAGE_KEY: &str = "GEOTHITE_FIELD_DIALOGUE_PAGE_V1";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedPage {
    version: u8,
    label: String,
    text: String,
    page_index: usize,
    visible_chars: usize,
}

/// Persist an ordinary script's later page on a checkpoint copy only.
/// Saves without that owner retain their existing representation. The live
/// state, script continuation and printer are never changed by saving.
pub(super) fn save(shell: &mut BevyRuntimeShell, path: &Path) -> Result<()> {
    let snapshot = shell.shell.presentation_snapshot()?;
    let page = if snapshot.battle.is_none()
        && shell.field_notice.is_none()
        && shell.pc_notice.is_none()
        && snapshot.ui.text_window_open
    {
        shell.field_text_reveal.as_ref().and_then(|reveal| {
            let text = snapshot.ui.text.as_ref()?;
            let pages = visible_field_dialog_pages(&snapshot, shell)?;
            (reveal.page_index > 0
                && reveal.page_index < pages.len()
                && reveal.text == pages.join("\u{1e}"))
            .then(|| SavedPage {
                version: 1,
                label: text.label.clone(),
                text: reveal.text.clone(),
                page_index: reveal.page_index,
                visible_chars: reveal.visible_chars,
            })
        })
    } else {
        None
    };
    let Some(page) = page else {
        return shell.shell.save(path);
    };
    let encoded = serde_json::to_string(&page).context("encode current field dialogue page")?;
    let mut checkpoint = shell.shell.clone();
    checkpoint
        .session_mut()
        .state_mut()
        .script_runtime
        .memory
        .insert(PAGE_KEY.into(), encoded);
    checkpoint.save(path)
}

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
    restore_page(shell)
}

fn restore_page(shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(encoded) = shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory
        .get(PAGE_KEY)
    else {
        return Ok(());
    };
    let page: SavedPage =
        serde_json::from_str(encoded).context("decode saved field dialogue page")?;
    let snapshot = shell.shell.presentation_snapshot()?;
    let text = snapshot
        .ui
        .text
        .as_ref()
        .context("saved field dialogue has no script text")?;
    let pages =
        visible_field_dialog_pages(&snapshot, shell).context("saved dialogue has no pages")?;
    ensure!(
        page.version == 1
            && snapshot.ui.text_window_open
            && snapshot.battle.is_none()
            && page.label == text.label
            && page.text == pages.join("\u{1e}")
            && page.page_index < pages.len()
            && page.visible_chars <= pages[page.page_index].chars().count(),
        "saved field dialogue page does not match its native text owner"
    );
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: page.text,
        page_index: page.page_index,
        visible_chars: page.visible_chars,
        frames_until_next_char: 0,
    });
    // The checkpoint envelope is consumed after rebuilding the production
    // printer. A later dialogue cannot inherit a stale saved page.
    shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .memory
        .remove(PAGE_KEY);
    Ok(())
}
