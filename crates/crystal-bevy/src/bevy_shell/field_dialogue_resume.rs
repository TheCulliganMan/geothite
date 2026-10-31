//! Rebuild the frontend cursor from an authoritative saved text boundary.
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use super::{
    BevyRuntimeShell, MenuCursor, VisibleFieldTextReveal, arm_visible_active_script_cursor,
    strict_readonly_cursor_index, visible_field_dialog_pages,
};
use crate::core::state::ScriptTextRuntimeKind;

const PAGE_KEY: &str = "GEOTHITE_FIELD_DIALOGUE_PAGE_V1";
const CHOICE_KEY: &str = "GEOTHITE_FIELD_YES_NO_V1";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedPage {
    version: u8,
    label: String,
    text: String,
    page_index: usize,
    visible_chars: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedChoice {
    version: u8,
    source_script: String,
    command_index: usize,
    selected: usize,
}

/// Persist an ordinary script's later page and yes/no choice on a checkpoint copy.
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
    let choice = shell
        .shell
        .session()
        .state()
        .script_runtime
        .pending_yes_no
        .as_ref()
        .map(|prompt| -> Result<SavedChoice> {
            Ok(SavedChoice {
                version: 1,
                source_script: prompt.source_script.clone(),
                command_index: prompt.command_index,
                selected: strict_readonly_cursor_index(&shell.yes_no_cursor, "ui:yes-no", 2)
                    .context("cannot save a yes/no prompt without its production cursor")?,
            })
        })
        .transpose()?;
    if page.is_none() && choice.is_none() {
        return shell.shell.save(path);
    }
    let mut checkpoint = shell.shell.clone();
    let memory = &mut checkpoint.session_mut().state_mut().script_runtime.memory;
    if let Some(page) = page {
        memory.insert(
            PAGE_KEY.into(),
            serde_json::to_string(&page).context("encode current field dialogue page")?,
        );
    }
    if let Some(choice) = choice {
        memory.insert(
            CHOICE_KEY.into(),
            serde_json::to_string(&choice).context("encode current field yes/no choice")?,
        );
    }
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
    restore_choice(shell)?;
    restore_page(shell)
}

fn restore_choice(shell: &mut BevyRuntimeShell) -> Result<()> {
    let memory = &shell.shell.session().state().script_runtime;
    let encoded = memory.memory.get(CHOICE_KEY);
    let Some(prompt) = memory.pending_yes_no.as_ref() else {
        ensure!(
            encoded.is_none(),
            "saved yes/no cursor has no active script owner"
        );
        return Ok(());
    };
    let selected = if let Some(encoded) = encoded {
        let choice: SavedChoice =
            serde_json::from_str(encoded).context("decode saved field yes/no choice")?;
        ensure!(
            choice.version == 1
                && choice.source_script == prompt.source_script
                && choice.command_index == prompt.command_index
                && choice.selected < 2,
            "saved yes/no cursor does not match its script owner"
        );
        choice.selected
    } else {
        // Older saves contain the script boundary but no frontend cursor.
        // Rebuild the same default YES used when that opcode first executes.
        0
    };
    shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "ui:yes-no".into(),
        option_index: selected,
    });
    shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .memory
        .remove(CHOICE_KEY);
    Ok(())
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
